include .env

db:
	docker run --rm -d --name postgres -p 5432:5432 \
  -e POSTGRES_DB=$(POSTGRES_DB) \
  -e POSTGRES_USER=$(POSTGRES_USER) \
  -e POSTGRES_PASSWORD=$(POSTGRES_PASSWORD) \
  postgres:latest


diesel:
	diesel migration run

diesel-test: diesel
	diesel migration run --migration-dir testing-migrations
stop:
	docker kill postgres

clippy:
	cargo clippy --all-features