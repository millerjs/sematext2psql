# sematext2psql

```
$ sematext2psql -h

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


Usage: sematext2psql [OPTIONS]

Options:
      --host <HOST>          [default: localhost]
      --user <USER>          [default: postgres]
      --port <PORT>          [default: 5432]
      --password <PASSWORD>  [default: password]
      --database <DATABASE>  [default: sematext]
  -h, --help                 Print help information
  -V, --version              Print version information
```
