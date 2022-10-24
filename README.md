# sematext2psql

![2022-10-24 08 55 56](https://user-images.githubusercontent.com/4062890/197543108-f9ad62f3-8690-4536-b9ab-3159a0b869c8.gif)

## download

Download the latest release here: https://github.com/millerjs/sematext2psql/releases/latest

## usage

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
