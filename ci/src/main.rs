#[tokio::main]
async fn main() -> eyre::Result<()> {
    let rust_image = "rustlang/rust:nightly";

    let client = dagger_sdk::connect().await?;

    let workdir = client.host().directory_opts(
        ".",
        dagger_sdk::HostDirectoryOpts {
            exclude: Some(vec!["target/", ".git/"]),
            include: None,
        },
    );

    let minio_url = "https://github.com/mozilla/sccache/releases/download/v0.3.3/sccache-v0.3.3-x86_64-unknown-linux-musl.tar.gz";

    // Main container
    let rust_base = client
        .container()
        .from(rust_image)
        .with_exec(vec!["apt-get", "update"])
        .with_exec(vec!["apt-get", "install", "--yes", "libpq-dev", "wget"])
        .with_exec(vec!["wget", minio_url])
        .with_exec(vec![
            "tar",
            "xzf",
            "sccache-v0.3.3-x86_64-unknown-linux-musl.tar.gz",
        ])
        .with_exec(vec![
            "mv",
            "sccache-v0.3.3-x86_64-unknown-linux-musl/sccache",
            "/usr/local/bin/sccache",
        ])
        .with_exec(vec!["chmod", "+x", "/usr/local/bin/sccache"])
        .with_env_variable("RUSTC_WRAPPER", "/usr/local/bin/sccache")
        .with_env_variable(
            "AWS_ACCESS_KEY_ID",
            std::env::var("AWS_ACCESS_KEY_ID").unwrap_or("".into()),
        )
        .with_env_variable(
            "AWS_SECRET_ACCESS_KEY",
            std::env::var("AWS_SECRET_ACCESS_KEY").unwrap_or("".into()),
        )
        .with_env_variable("SCCACHE_BUCKET", "sccache")
        .with_env_variable("SCCACHE_REGION", "auto")
        .with_env_variable("SCCACHE_ENDPOINT", "https://api-minio.front.kjuulh.io")
        .with_exec(vec!["cargo", "install", "cargo-chef"])
        .with_exec(vec!["cargo", "install", "cargo-leptos"])
        .with_workdir("/app");

    let exit_code = rust_base.exit_code().await?;
    if exit_code != 0 {
        eyre::bail!("could not build base");
    }

    let rust_prepare = rust_base
        .with_mounted_directory(".", workdir.id().await?)
        .with_exec(vec![
            "cargo",
            "chef",
            "prepare",
            "--recipe-path",
            "recipe.json",
        ]);

    let exit_code = rust_prepare.exit_code().await?;
    if exit_code != 0 {
        eyre::bail!("could not build prepare");
    }

    let rust_cacher = rust_base
        .with_exec(vec!["apt", "update"])
        .with_exec(vec![
            "apt",
            "install",
            "pkg-config",
            "openssl",
            "libssl-dev",
            "-y",
        ])
        .with_exec(vec!["rustup", "target", "add", "wasm32-unknown-unknown"])
        .with_file(
            "/recipe.json",
            rust_prepare.file("./recipe.json").id().await?,
        )
        .with_mounted_directory(".", workdir.id().await?)
        .with_exec(vec![
            "cargo",
            "chef",
            "cook",
            "--release",
            "--recipe-path",
            "/recipe.json",
        ]);

    let exit_code = rust_cacher.exit_code().await?;
    if exit_code != 0 {
        eyre::bail!("could not build cacher");
    }

    let rust_builder = rust_base
        .with_exec(vec![
            "curl",
            "-sL",
            "https://deb.nodesource.com/setup_12.x",
            "-o",
            "/node_12.txt",
        ])
        .with_exec(vec!["chmod", "+x", "/node_12.txt"])
        .with_exec(vec!["bash", "-c", "/node_12.txt"])
        .with_exec(vec!["apt-get", "update"])
        .with_exec(vec!["apt-get", "install", "nodejs"])
        .with_mounted_directory(".", workdir.id().await?)
        .with_directory(
            "/app/target",
            rust_cacher.directory("/app/target").id().await?,
        )
        .with_directory(
            "/usr/local/cargo",
            rust_cacher.directory("/usr/local/cargo").id().await?,
        )
        .with_exec(vec!["rustup", "target", "add", "wasm32-unknown-unknown"])
        .with_exec(vec!["npm", "install", "-g", "sass"])
        .with_env_variable("LEPTOS_BROWSERQUERY", "defaults")
        .with_exec(vec!["cargo", "leptos", "build", "--release"]);

    let exit_code = rust_builder.exit_code().await?;
    if exit_code != 0 {
        eyre::bail!("could not build builder");
    }

    let tag = chrono::Utc::now().timestamp();

    let prod_image = "gcr.io/distroless/cc-debian11";
    let prod = client
        .container()
        .from(prod_image)
        .with_workdir("/app")
        .with_directory("/app", rust_builder.directory("/app/target").id().await?)
        .with_env_variable("LEPTOS_SITE_ADDRESS", "0.0.0.0:3000")
        .with_entrypoint(vec!["./server/release/ssr_mode_axum"]);

    prod.publish(format!("docker.io/kasperhermansen/bitebuds:{tag}"))
        .await?;

    Ok(())
}
