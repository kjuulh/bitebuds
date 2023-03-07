use domain::{Event, Image};
use inquire::validator::ValueRequiredValidator;
use regex::Regex;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    color_eyre::install()?;

    let cli = clap::Command::new("biteme")
        .subcommand(clap::Command::new("generate").subcommand(clap::Command::new("article")));

    let args = std::env::args();

    let matches = cli.get_matches_from(args);
    match matches.subcommand() {
        Some(("generate", subm)) => match subm.subcommand() {
            Some(("article", _subm)) => {
                generate_article().await?;
            }
            _ => panic!("command not valid"),
        },
        _ => panic!("command not valid"),
    }

    Ok(())
}

async fn generate_article() -> eyre::Result<()> {
    let name = inquire::Text::new("What are you going to eat?")
        .with_validator(ValueRequiredValidator::default())
        .prompt()?;
    let description = inquire::Editor::new("Do you want to provide a description?")
        .prompt_skippable()?
        .and_then(|ci| if ci == "" { None } else { Some(ci) });
    let time = inquire::DateSelect::new("When is the event?")
        .with_min_date(chrono::Local::now().date_naive())
        .prompt()?;
    let cover_image = inquire::Text::new("Do you have a picture for it?")
        .prompt_skippable()?
        .and_then(|ci| if ci == "" { None } else { Some(ci) });
    let cover_alt = if let Some(_) = cover_image {
        Some(
            inquire::Text::new("Do you have a description for the image?")
                .with_validator(ValueRequiredValidator::default())
                .prompt()?,
        )
    } else {
        None
    };

    let prepared_name = name.replace(" ", "-");
    let prepared_name = prepared_name.replace("--", "-");
    let prepared_name = prepared_name.trim_matches('-');

    let re = Regex::new(r"[a-zA-Z-_0-9]*")?;
    let name_slug = re
        .find_iter(&prepared_name)
        .map(|n| n.as_str())
        .collect::<Vec<_>>();
    let name_slug = name_slug.join("");

    let slug = format!("{}-{}", time.format("%Y-%m-%d"), name_slug.to_lowercase());

    let event = Event {
        id: uuid::Uuid::new_v4(),
        cover_image: cover_image.zip(cover_alt).map(|(image, alt)| Image {
            id: uuid::Uuid::new_v4(),
            url: image,
            alt,
            metadata: None,
        }),
        name,
        description: description.clone(),
        time,
        recipe_id: None,
        images: Vec::new(),
        metadata: None,
    };

    let contents = serde_yaml::to_string(&event)?;
    let contents = format!(
        "---
{}---

{}",
        contents,
        description.unwrap_or("".into())
    );

    tokio::fs::write(format!("articles/events/{}.md", slug), contents).await?;

    Ok(())
}
