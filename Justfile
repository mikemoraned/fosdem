model_dir := "./shared/data/model"
years := "2025 2026"
current_year := "2026"
schedule_file := "./content/schedule/" + current_year + ".xml"
assets_dir := "./assets"

fresh_test:
    cargo clean
    cargo build --release
    cargo test --release

fetch_schedules:
    for year in {{years}}; do \
        wget -O ./content/schedule/$year.xml https://fosdem.org/$year/schedule/xml; \
    done

import_schedule:
    mkdir -p {{model_dir}}
    cargo run --bin import_events --release -- --pentabarf {{schedule_file}} --model-dir {{model_dir}}

index_next: embeddings_next related_next
    
embeddings_next:
    RUST_LOG=info cargo run --bin fetch_openai_embeddings --release -- --model-dir {{model_dir}}

related_next:
    RUST_LOG=info cargo run --bin generate_related --release -- --model-dir {{model_dir}} --limit 5 --json {{assets_dir}}/all.limit5.json

bring_up_to_date: fetch_schedules import_schedule index_next

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