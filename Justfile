model_dir := "./shared/data/model"
years := "2024 2025 2026"
current_year := "2026"
pentabarf_dir := "./content/schedule"
assets_dir := "./assets"
blog_content_dir := "./blog/content/posts"

export OPENAI_API_KEY := `op read "op://Dev/fosdem-local-openai-key/password"`

fresh_test:
    cargo clean
    cargo build --release
    cargo test --release

fetch_schedules:
    for year in {{years}}; do \
        wget -O {{pentabarf_dir}}/$year.xml https://fosdem.org/$year/schedule/xml; \
    done

import_schedules:
    mkdir -p {{model_dir}}
    RUST_LOG=debug cargo run --bin import_events --release -- --pentabarf-dir {{pentabarf_dir}} --years "{{years}}" --model-dir {{model_dir}}

index_next: embeddings_next related_next
    
embeddings_next:
    RUST_LOG=info cargo run --bin fetch_openai_embeddings --release -- --model-dir {{model_dir}}
    gzip -9v  --force {{model_dir}}/embeddings.json

related_next:
    RUST_LOG=info cargo run --bin generate_related --release -- --model-dir {{model_dir}} --years "{{current_year}}" --limit 5 --json {{assets_dir}}/all.limit5.json

bring_up_to_date: fetch_schedules import_schedules index_next
    cargo test -p shared --test integration_tests
    RUST_LOG=info cargo run --bin add_data_post --release -- --model-dir {{model_dir}} --blog-content-dir {{blog_content_dir}}

webapp:
    RUST_LOG=debug cargo run --bin fly -- --model-dir {{model_dir}} --blog-content-dir {{blog_content_dir}} --current-year {{current_year}} --selectable-years "{{years}}"

test_webapp:
    cargo test -p webapp --test integration_tests

deploy_staging: deploy_staging_secrets deploy_staging_app test_staging

deploy_staging_secrets:
    fly secrets deploy --config fly.staging.toml

deploy_staging_app:
    fly deploy --config fly.staging.toml

test_staging:
    TEST_BASE_URL=https://fosdem2024-staging.fly.dev cargo test -p webapp --test integration_tests

deploy_prod: deploy_prod_secrets deploy_prod_app test_prod

deploy_prod_secrets:
    fly secrets deploy --config fly.prod.toml

deploy_prod_app:
    fly deploy --config fly.prod.toml

test_prod:
    TEST_BASE_URL=https://fosdem.houseofmoran.io cargo test -p webapp --test integration_tests 
