use anyhow::Result;
use dotenv::dotenv;
use html_stripper::strip;
use once_cell::sync::Lazy;
use openai::{
    chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole},
    set_key,
};
use scraper::{Html, Selector};
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::{
    env,
    io::{stdin, stdout, Write},
};

#[derive(Debug, Serialize, Deserialize)]
enum JobType {
    #[serde(rename = "Vollzeit")]
    FullTime,
    #[serde(rename = "Teilzeit")]
    PartTime,
    #[serde(rename = "Gerinfügig")]
    Marginal,
    #[serde(rename = "Praktika")]
    Internship,
    #[serde(rename = "Freelancer*in, Projektarbeit")]
    Freelance,
    #[serde(rename = "Lehre")]
    Apprenticeship,
    #[serde(rename = "Diplomarbeit, Dissertation")]
    DiplomaThesis,
}

#[derive(Debug, Serialize, Deserialize)]
struct Contact {
    name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    linkedin: Option<String>,
    github: Option<String>,
    portfolio: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Listing {
    manual_listing_info: ManualListingInfo,
    ai_listing_info: AIListingInfo,
}

#[derive(Debug, Serialize, Deserialize)]
struct ManualListingInfo {
    company: String,
    title: String,
    job_type: Vec<JobType>,
    locations: Vec<String>,
    application_link: String,
    karriere_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AIListingInfo {
    description: Option<String>,
    salary_min: Option<u32>,
    salary_max: Option<u32>,
    contact: Option<Contact>,
    perks: Option<Vec<String>>,
    requirements: Option<Vec<String>>,
    responsibilities: Option<Vec<String>>,
}

const KARRIERE_LINK: &str = "https://www.karriere.at/jobs/6759197";

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;
    println!("OPENAI API KEY: {}", env::var("OPENAI_KEY")?);

    set_key(env::var("OPENAI_KEY")?);

    let mut messages = vec![ChatCompletionMessage {
        role: ChatCompletionMessageRole::System,
        content: "You extract information from Job listings".to_string(),
        name: None,
    }];

    // loop {
    //     print!("User: ");
    //     stdout().flush()?;

    //     let mut user_message_content = String::new();

    //     stdin().read_line(&mut user_message_content)?;
    //     messages.push(ChatCompletionMessage {
    //         role: ChatCompletionMessageRole::User,
    //         content: user_message_content,
    //         name: None,
    //     });

    //     let chat_completion = ChatCompletion::builder("gpt-3.5-turbo", messages.clone())
    //         .create()
    //         .await?;
    //     let returned_message = chat_completion.choices.first().unwrap().message.clone();

    //     println!(
    //         "{:#?}: {}",
    //         &returned_message.role,
    //         &returned_message.content.trim()
    //     );

    //     messages.push(returned_message);
    // }

    let html = reqwest::ClientBuilder::new()
        .user_agent("Mozilla/5.0")
        .build()?
        .get(KARRIERE_LINK)
        .send()
        .await?
        .text()
        .await?;

    let html = Html::parse_document(&html);

    scrape_info(&html);

    Ok(())
}

fn scrape_info(html: &Html) {
    let title = html
        .select(&TITLE_SELECTOR)
        .next()
        .unwrap()
        .text()
        .next()
        .unwrap();

    let company_name = html
        .select(&COMPANY_NAME_SELECTOR)
        .next()
        .unwrap()
        .text()
        .next()
        .unwrap();

    let locations = html
        .select(&LOCATIONS_SELECTOR)
        .next()
        .unwrap()
        .value()
        .attr("data-job-locations")
        .unwrap()
        .split(',')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    let employment_types = html
        .select(&EMPLOYMENT_TYPES_SELECTOR)
        .next()
        .unwrap()
        .text()
        .next()
        .unwrap()
        .split(',')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    println!("Title: {}", title);
    println!("Company Name: {}", company_name);
    println!("Locations: {:?}", locations);
    println!("Employment Types: {:?}", employment_types);
}

static TITLE_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("h1.m-jobHeader__jobTitle").unwrap());

static COMPANY_NAME_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("a.m-jobHeader__companyLink").unwrap());

static LOCATIONS_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("li.jobHeader__jobLocations").unwrap());

static EMPLOYMENT_TYPES_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("li.m-jobHeader__jobEmploymentTypes").unwrap());
