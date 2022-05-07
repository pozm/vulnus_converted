use clap::{Parser, Subcommand, ArgEnum};
use reqwest::header::HeaderValue;
use serde_json::Error;
use std::{path::PathBuf, fs::{read_dir, self, File, OpenOptions}, io::Read, str::FromStr};
use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapData {
    #[serde(rename(serialize="artist",deserialize = "_artist"))]
    pub artist: Option<String>,
    #[serde(skip_serializing)]
    #[serde(rename(deserialize="_difficulties",serialize="difficulties"))]
    pub difficulties_location: Vec<String>,
    #[serde(rename(deserialize="_mappers",serialize="mappers"))]
    pub mappers: Vec<String>,
    #[serde(rename(deserialize="_music",serialize="music"))]
    pub music: String,
    #[serde(rename(deserialize="_title",serialize="title"))]
    pub title: String,
    #[serde(rename(deserialize="_version",serialize="version"))]
    pub version: i64,

    // #[serde(skip_deserializing)]
    // #[serde(rename="difficulties")]
    // pub difficulties_data: Vec<MapDataDifficulty>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapDataDifficulty {
    #[serde(rename(deserialize = "_approachDistance",serialize="approachDistance"))]
    pub ad: i32,
    #[serde(rename(deserialize = "_approachTime",serialize="approachTime"))]
    pub at: i32,
    #[serde(rename(deserialize = "_name",serialize="name"))]
    pub name: String
}

#[derive(ArgEnum,Debug,Clone)]
enum AppModes {
    Default,
    Single,
    Online
}

#[derive(ArgEnum,Debug,Clone)]
enum OutputModes {
    Json,
    Csv
}

impl FromStr for AppModes {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "default" => Ok(AppModes::Default),
            "single" => Ok(AppModes::Single),
            "online" => Ok(AppModes::Online),
            _ => Err(format!("Unknown mode: {}", s))
        }
    }
}

impl FromStr for OutputModes {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "csv" => Ok(Self::Csv),
            "json" => Ok(Self::Json),
            _ => Err(format!("Unknown mode: {}", s))
        }
    }
}

#[derive(Parser,Debug)]
#[clap(author, version, about, long_about = None)]
struct AppArgs {
    #[clap(parse(from_os_str))]
    dir: PathBuf,
    #[clap(short,default_value="default")]
    mode: AppModes,
    #[clap(short,default_value="csv")]
    output: OutputModes,

    #[clap(subcommand)]
    command: Option<ProgramCommands>,
}

#[derive(Subcommand,Debug)]
enum ProgramCommands {
    Online(OnlineCmd)
}
#[derive(clap::Args,Debug)]
struct OnlineCmd {
    #[clap(short)]
    token: String,
    #[clap(short,default_value  = "966094481466216489")] // id to maps channel
    channel_id : u64,
}

fn get_data_from_meta(path: &PathBuf) -> Result<MapData, String> {
    let file = File::open(path).or(Err(format!("Could not open file: {}", path.to_str().unwrap())))?;
    let unsafe_map_data : Result<MapData,Error> = serde_json::from_reader(file);
    if let Ok(mut mapdata) = unsafe_map_data {

        // for difficulty in &mapdata.difficulties_location {
        //     let diff_file = File::open(path.join("..").join(difficulty)).or(Err(format!("Could not open diff file: {} | {}", path.to_str().unwrap(),difficulty)))?;
        //     let unsafe_diff_data : Result<MapDataDifficulty,Error> = serde_json::from_reader(diff_file);
        //     if let Ok(diff_data) = unsafe_diff_data {
        //         mapdata.difficulties_data.push(diff_data)
        //     } else {
        //         return Err(format!("Could not parse diff file: {} | {}", path.to_str().unwrap(),difficulty));
        //     }
        // }
        // println!("{:?}", mapdata);
        Ok(mapdata)
    } else {
        Err(format!("Could not parse file: {}", path.to_str().unwrap()))
    }
}

fn add_to_list(list: &mut Vec<MapData>, path: &PathBuf) -> Result<(), String> {
    let map_data = get_data_from_meta(&path.join("meta.json"))?;
    list.push(map_data);
    Ok(())
}

#[derive(Serialize)]

struct CsvRowData {
    title: String,
    artist: String,
    mappers: String,
    difficulties_amount: i32,
    version: i64,
}


fn main() {
    let args = AppArgs::parse();


    println!("{:?}", args);


    let mut map_data : Vec<MapData> = vec![];

    match args.mode {
        AppModes::Default => {
            let files = fs::read_dir(&args.dir).expect("unable to read the path you provided");
            for file in files {
                let file = file.expect("unable to read the path you provided");
                if file.path().is_dir() {
                    if let Err(er) =  add_to_list(&mut map_data, &file.path()) {
                        println!("Failed to add {} | Reasoning : {}", file.path().to_str().unwrap(),er);
                    } else {
                        // println!("Added {}", file.path().to_str().unwrap());
                    }
                }
            }
        }
        AppModes::Single => todo!(),
        AppModes::Online => {
            match args.command.unwrap() {
                ProgramCommands::Online(online_args) => {
                    let mut headers = reqwest::header::HeaderMap::new();
                    headers.insert("authorization", HeaderValue::from_str(&online_args.token).expect("unable to get token"));
                    let mut client = reqwest::ClientBuilder::new().default_headers(headers).build().expect("unable to build client");


                    client.get("");

                }
                _=>unreachable!()
            }
        },
    }


    // println!("got mapdata: {:#?}", map_data);


    match args.output {
        OutputModes::Json => {
            let mut output_csv_file = OpenOptions::new().create(true).truncate(true).write(true).open("./output.json").expect("unable to make csv file");

            if let Err(er) = serde_json::to_writer(&output_csv_file, &map_data) {
                println!("Failed to write json file | Reasoning : {}", er);
            } else {
                println!("Successfully wrote json file");
            }
        }
        OutputModes::Csv => {
            let mut output_csv_file = OpenOptions::new().create(true).truncate(true).write(true).open("./output.csv").expect("unable to make csv file");
            let mut writer = csv::Writer::from_writer(&output_csv_file);

            for map in map_data {
                if let Err(er) = writer.serialize(CsvRowData {
                    title: map.title,
                    artist: map.artist.unwrap_or_else(|| "".to_string()),
                    mappers: map.mappers.join(","),
                    difficulties_amount: map.difficulties_location.len() as i32,
                    version: map.version,
                }) {
                    println!("Failed to write csv row | Reasoning : {}", er);
                } else {
                    println!("Successfully wrote csv row");
                }
            }   
            // println!("{:?}",writer.into_inner().unwrap())
        },
    }

}