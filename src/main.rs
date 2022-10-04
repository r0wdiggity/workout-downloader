use {
    chrono::offset::Local,
    hyper::{Client, http, Body, Request, Method},
    hyper::client::HttpConnector,
    hyper_tls::HttpsConnector,
    serde::{Deserialize, Serialize},
    std::env,
    std::fs::File,
    std::io::prelude::*,
};


#[derive(Debug, Serialize, Deserialize)]
pub struct Workout {
    pub id: i32,
    #[serde(rename = "type")]
    pub type_name: String,
    pub name: String
}

static BASE: &str = "https://intervals.icu";


fn get_today() -> String {
    let dt = Local::now();
    format!("{}", dt.format("%Y-%m-%d"))
}

fn parse_json(json: &str) -> (String, String) {
    let workouts: Vec<Workout> = serde_json::from_str(json).expect("Could not parse workouts json");
    for workout in workouts {
        if workout.type_name == "Ride" || workout.type_name == "Virtual Ride" {
            return (workout.id.to_string(), workout.name)
        }
    }
    ("".to_string(), "".to_string())
}

fn build_uri(endpoint: &str) -> http::Uri {
    let uri= format!("{}{}", BASE, endpoint);
    uri.parse::<hyper::Uri>().expect("Failed to parse URI")
}

fn build_body() -> Body {
    Body::from("test")
}

fn build_request(uri: http::Uri, body: Body, method: hyper::Method) -> Request<Body> {
    let api_key = env::var("INTERVALS_TOKEN").expect("Please set your INTERVALS_TOKEN environment variable to your API Key");
    let auth_token = base64::encode(format!("{}:{}", "API_KEY", api_key));
    let auth_header = format!("Basic {}", auth_token);
    Request::builder()
        .method(method)
        .header("Authorization", auth_header)
        .uri(uri)
        .body(body)
        .expect("Failed to build request")
}

async fn get_workout_id(client: &Client<HttpsConnector<HttpConnector>>, user_id: &str) -> Result<(String, String), Box<dyn std::error::Error + Send + Sync>>{
    let endpoint = format!("/api/v1/athlete/{}/events?oldest={}&newest={}", user_id, get_today(), get_today());
    let req = build_request(build_uri(&endpoint), build_body(), Method::GET);
    println!("{:?}",build_uri(&endpoint));
    let result = client.request(req).await?;

    let body = hyper::body::to_bytes(result.into_body()).await?;
    let body_string = String::from_utf8(body.to_vec()).unwrap();

    Ok(parse_json(&body_string))
}

async fn get_workout(client: &Client<HttpsConnector<HttpConnector>>, workout_id: &str, user_id: &str) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>  {
    let ext = "zwo";
    let endpoint = format!("/api/v1/athlete/{}/events/{}/download{}", user_id, workout_id, ext);
    let req = build_request(build_uri(&endpoint), build_body(), Method::POST);

    let result = client.request(req).await?;

    let workout = hyper::body::to_bytes(result.into_body()).await?;

    Ok(workout.to_vec())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let path = "/Users/anthonygiovannelli/Documents/Zwift/Workouts/2750322/today.zwo";
    let user_id = env::var("INTERVALS_ID").expect("Please set the INTERVALS_ID environment variable to your Athlete ID");
    println!("Todays date is: {} ... ", get_today());
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    println!("Querying intervals.icu for today's workouts ... ");

    let (workout_id, workout_name) = get_workout_id(&client, &user_id).await?;

    if workout_id.is_empty() {
        println!("No workout");
        return Ok(())
    }

    println!("Attempting to download workout - {} ... ", workout_name);

    let workout = get_workout(&client, &workout_id, &user_id).await?;

    let vec_ref = &workout;
    let bytes: &[u8] = vec_ref.as_ref();

    println!("Writing workout {} to file ... ", workout_name);

    let mut file = File::create(path).expect("Failed to create file");

    let _ = file.write_all(bytes);
    Ok(())
}
