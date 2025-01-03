model_dir := "./shared/data/model"
year := "2025"
schedule_file := "./content/schedule/" + year + ".xml"
assets_dir := "./assets"

fetch_schedule:
    wget -O {{schedule_file}} https://fosdem.org/{{year}}/schedule/xml

import_schedule:
    mkdir -p {{model_dir}}
    cargo run --bin import_events --release -- --pentabarf {{schedule_file}} --model-dir {{model_dir}}

index_next: embeddings_next related_next
    
embeddings_next:
   cargo run --bin fetch_openai_embeddings --release -- --model-dir {{model_dir}}

related_next:
    cargo run --bin generate_related --release -- --model-dir {{model_dir}} --limit 5 --json {{assets_dir}}/all.limit5.json

bring_up_to_date: fetch_schedule import_schedule index_next

webapp:
    RUST_LOG=info cargo run --bin fly -- --model-dir {{model_dir}}

deploy_staging: deploy_staging_secrets deploy_staging_app

deploy_staging_secrets:
    fly secrets deploy --config fly.staging.toml

deploy_staging_app:
    fly deploy --config fly.staging.toml

deploy_prod: deploy_prod_secrets deploy_prod_app

deploy_prod_secrets:
    fly secrets deploy --config fly.prod.toml

deploy_prod_app:
    fly deploy --config fly.prod.toml