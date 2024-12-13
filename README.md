
# MY ACTIX WEB STARTER
usage for makes your actix initiation fast

## USAGE 
1. Check the cargo sqlx and docker in your enviroment

    ``` bash
    cargo --version
    sqlx --version
    docker --version
    ```
    If you dont have it you cant install cargo rustc and sqlx in this documentation
    - [cargo & rust](https://doc.rust-lang.org/cargo/getting-started/installation.html)
    - [sqlx for rust](https://crates.io/crates/sqlx-cli)
    - [docker](https://docs.docker.com/engine/install/)
2. clone this repository

``` bash 
git clone https://github.com/Reihan1305/actix_web_starter.git
```

3. Edit env.example 
```bash
POSTGRES_USER=your pg username
POSTGRES_PASSWORD=your pg password
POSTGRES_DB=your db name
DATABASE_URL= your db url
REDIS_HOSTNAME=your redis url
RABBITMQ_URL=your rabbitmq url
#optional
REDIS_PASSWORD= if your redis have password
```

4. runing docker compose for database (Optional)
if you want run postgres from docker you can run docker compose
```bash
docker compose up -d
```

5. runing migration
for runing migration run this commant
```bash
sqlx migrate run
```

6. runing cargo build and run
run this commmand for build and runing actix web 
```bash
cargo build run
```

### optional
if you dont want use auth you cant delete this file
```bash
/src/midleware/authmiddleware.rs
/src/modules/auth
```
or if you want use auth you  cant uncommand mod file 
```
/src/middleware/mod.rs
```
and     
```
src/modules/mod.rs
```

## add your own table
- create migration file
run this command for create migration up and down file
```bash
sqlx migration add -r <migration name>
```

- modify up and down migration file

- run migration
```bash
sqlx migration run 
```

## Features

- redis service
- postgres using sqlx
- rabbitmq connection

## Authors

- [@Reihan1305](https://www.github.com/Reihan1305)

