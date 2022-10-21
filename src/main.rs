use chrono::{DateTime, NaiveDateTime};
use clap::Parser;
use postgres::types::{ToSql};
use postgres::{Client, NoTls};
use std::io::{self, BufRead};

#[derive(Parser, Debug)]
#[command(author, version, about = r#"
Reads json logs from stdin and imports them into postgres for analysis.

Helpful usage flow:

  To install the `sematext2psql` executable:
  
      cargo install --path .

  To setup the database:
  
      psql -c 'create database sematext;'
  
  To download and extract logs from s3 (OSX brew example):
      
      brew install lz4 liblzf
      
      mkdir sematext_logs; cd sematext_logs/
      
      aws s3 cp --recursive s3://benchprep-sematext-production/sematext_52f4930f/2022/10/21/12/ ./
      aws s3 cp --recursive s3://benchprep-sematext-production/sematext_52f4930f/2022/10/21/13/ ./
  
      lzf -d *.json.lzf
    
      rg '"name":"sidekiq-' > filtered_logs.json
  
  To import into postgres
  
      sematext2psql < filtered_logs.json
"#)]
struct Args {
    #[arg(long, default_value = "localhost")]
    host: String,
    #[arg(long, default_value = "postgres")]
    user: String,
    #[arg(long, default_value = "5432")]
    port: String,
    #[arg(long, default_value = "password")]
    password: String,
    #[arg(long, default_value = "sematext")]
    database: String,
}

static BUFFER_SIZE: usize = 10_000;

static SETUP_SQL: &'static str = "
    CREATE TABLE IF NOT EXISTS sematext_logs (
       pod_name text NOT NULL,
       message text,
       route text,
       created_at TIMESTAMP NOT NULL
    );
    
    create index IF NOT EXISTS sematext_logs_pod_name on sematext_logs (pod_name);
    create index IF NOT EXISTS sematext_logs_created_at on sematext_logs (created_at);
    create index IF NOT EXISTS sematext_logs_route on sematext_logs (route);
";

#[derive(Debug)]
struct SematextLog {
    pod_name: String,
    message: String,
    created_at: NaiveDateTime,
}

struct BufferImporter {
    client: Client,
    buffer: Vec<SematextLog>
}

impl BufferImporter {
    fn connect(args: &Args) -> Self {
        let url = format!("postgresql://{}:{}@{}:{}/{}", args.user, args.password, args.host, args.port, args.database);
            
        Self {
            buffer: Vec::with_capacity(BUFFER_SIZE),
            client: Client::connect(&*url, NoTls).expect("could not setup database connection")
        }
    }

    fn setup_tables(&mut self)  {
        self.client.batch_execute(SETUP_SQL).expect("could not setup database")
    }
    
    fn write(&mut self, log: SematextLog) {
        self.buffer.push(log);
        if self.buffer.len() >= BUFFER_SIZE {
            self.flush();
        }        
    }
    
    fn flush(&mut self) {
        let mut params = Vec::<&(dyn ToSql + Sync)>::with_capacity(self.buffer.len());
        let mut bind_params = Vec::with_capacity(self.buffer.len());

        for (i, log) in self.buffer.iter().enumerate() {
            bind_params.push(format!("(${},${},${})", i*3+1, i*3+2, i*3+3));
            params.extend_from_slice(&[&log.pod_name, &log.message, &log.created_at]);
        }
        
        let query = format!("
           insert into sematext_logs(pod_name, message, created_at) values {}
        ", bind_params.join(","));
        
        self.client.execute(&query, &params[..]).expect("Failed to insert buffer");
        self.buffer.clear();
    }
}

fn parse_line(line: &str) -> SematextLog {
    let log_json = line.split(".json:").nth(1).ok_or(line).unwrap();
    let parsed = json::parse(log_json).expect(&*format!("invalid json: {}", log_json));
    let timestamp = parsed["@timestamp"].as_str().expect("missing timestamp");
    
    SematextLog {
        pod_name: parsed["kubernetes"]["pod"]["name"].to_string(),
        message: parsed["message"].to_string(),
        created_at: DateTime::parse_from_rfc3339(timestamp).expect("bad timestamp").naive_utc()
    }
}

fn start_progress() -> indicatif::ProgressBar {
    let progress = indicatif::ProgressBar::new_spinner();
    progress.enable_steady_tick(std::time::Duration::from_millis(120));
    progress.set_style(indicatif::ProgressStyle::with_template("{spinner:.blue} [{pos}] {msg}").unwrap());
    progress.set_message("Reading stdin...");
    progress
}

fn main() {
    let mut buffer = BufferImporter::connect(&Args::parse());
    let progress = start_progress();

    buffer.setup_tables();
    
    for line in io::stdin().lock().lines() {
        let log = parse_line(&line.expect("Could not read line from standard in"));
        buffer.write(log);
        progress.inc(1);
    }
    
    buffer.flush();
    
    progress.finish_with_message("Done.");
}
