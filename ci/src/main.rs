#[tokio::main]
async fn main() -> eyre::Result<()> {
    let _ = dotenv::dotenv();
    let rust_image = "docker.io/rustlang/rust:nightly";

    let client = dagger_sdk::connect().await?;

    let workdir = client.host().directory_opts(
        ".",
        dagger_sdk::HostDirectoryOpts {
            exclude: Some(vec!["target/", ".git/"]),
            include: None,
        },
    );

    let minio_url = "https://github.com/mozilla/sccache/releases/download/v0.3.3/sccache-v0.3.3-x86_64-unknown-linux-musl.tar.gz";

    let sccache_download_cache = client.cache_volume("sccache_download");
    let cargo_cache = client.cache_volume("cargo_cache");

    // Main container
    let rust_base = client
        .container()
        .from(rust_image)
        .with_exec(vec!["apt-get", "update"])
        .with_exec(vec!["apt-get", "install", "--yes", "libpq-dev", "wget"])
        .with_exec(vec!["mkdir", "-p", "/src/downloads"])
        .with_workdir("/src/downloads")
        .with_mounted_cache("/src/downloads", sccache_download_cache.id().await?)
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
        .with_mounted_cache("~/.cargo", cargo_cache.id().await?)
        .with_exec(vec!["rustup", "target", "add", "wasm32-unknown-unknown"])
        .with_exec(vec!["cargo", "install", "cargo-chef"])
        .with_exec(vec!["cargo", "install", "cargo-leptos"])
        .with_workdir("/app");

    let exit_code = rust_base.exit_code().await?;
    if exit_code != 0 {
        eyre::bail!("could not build base");
    }

    let target_cache = client.cache_volume("target_cache");
    let rust_prepare = rust_base
        .with_mounted_directory(".", workdir.id().await?)
        .with_mounted_cache("target", target_cache.id().await?)
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

    let target_rust_cache = client.cache_volume("target_rust_cache");

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
        .with_mounted_cache("target", target_rust_cache.id().await?)
        .with_exec(vec![
            "cargo",
            "chef",
            "cook",
            "--release",
            "--recipe-path",
            "/recipe.json",
        ])
        .with_mounted_directory(".", workdir.id().await?);

    let exit_code = rust_cacher.exit_code().await?;
    if exit_code != 0 {
        eyre::bail!("could not build cacher");
    }

    let nodejs_cacher = client.cache_volume("node");

    // something

    let rust_pre_builder = rust_cacher
        .with_exec(vec!["mkdir", "-p", "node"])
        .with_mounted_cache("node", nodejs_cacher.id().await?)
        .with_exec(vec![
            "curl",
            "-sL",
            "https://deb.nodesource.com/setup_12.x",
            "-o",
            "node/node_12.txt",
        ])
        .with_exec(vec!["chmod", "+x", "node/node_12.txt"])
        .with_exec(vec!["bash", "-c", "node/node_12.txt"])
        .with_exec(vec!["apt-get", "update"])
        .with_exec(vec!["apt-get", "install", "nodejs"])
        .with_exec(vec!["npm", "install", "-g", "sass"]);

    let exit_code = rust_pre_builder.exit_code().await?;
    if exit_code != 0 {
        eyre::bail!("could not build rust_pre_builder");
    }

    let rust_builder = rust_pre_builder
        .with_env_variable("LEPTOS_BROWSERQUERY", "defaults")
        .with_exec(vec!["cargo", "leptos", "build", "--release"]);

    let exit_code = rust_builder.exit_code().await?;
    if exit_code != 0 {
        eyre::bail!("could not build builder");
    }

    let tag = chrono::Utc::now().timestamp();

    let prod_image = "debian:bullseye";
    let prod = client
        .container()
        .from(prod_image)
        .with_exec(vec!["apt", "update"])
        .with_exec(vec!["apt", "install", "-y", "zlib1g", "git"])
        .with_workdir("/app")
        .with_file(
            "/app/ssr_modes",
            rust_builder
                .file("/app/target/server/release/ssr_modes")
                .id()
                .await?,
        )
        .with_directory(
            "/app/site",
            rust_builder.directory("/app/target/site").id().await?,
        )
        .with_env_variable("LEPTOS_OUTPUT_NAME", "ssr_modes")
        .with_env_variable("LEPTOS_SITE_ROOT", "site")
        .with_env_variable("LEPTOS_SITE_PKG_DIR", "pkg")
        .with_env_variable("LEPTOS_SITE_ADDR", "0.0.0.0:3000")
        .with_env_variable("LEPTOS_RELOAD_PORT", "3001")
        .with_entrypoint(vec!["/app/ssr_modes"]);

    let image_tag = format!("docker.io/kasperhermansen/bitebuds:{tag}");
    prod.publish(&image_tag).await?;

    let update_deployment = client
        .container()
        .from("docker.io/kasperhermansen/update-deployment:1680548342")
        .with_env_variable("GIT_USERNAME", "kjuulh")
        .with_env_variable("GIT_PASSWORD", std::env::var("GIT_PASSWORD").unwrap())
        .with_exec(vec![
            "update-deployment",
            "--repo",
            "https://git.front.kjuulh.io/kjuulh/bitebuds-deployment.git",
            "--service",
            "bitebuds",
            "--image",
            &image_tag,
        ])
        .exit_code()
        .await?;

    Ok(())
}
