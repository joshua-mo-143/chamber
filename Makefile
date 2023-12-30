dshuttle:
	docker exec -it shuttle_chamber-server_shared_postgres psql -U postgres -h localhost -p 5432

dt:
	docker run -d -t -p 8500:5432 -e POSTGRES_PASSWORD=postgres --name chamber postgres && sqlx migrate run --source chamber-server/migrations --database-url postgres://postgres:postgres@localhost:8500/postgres

dtu:
	make dt && cargo test --test postgres_tests --no-default-features

dtr:
	docker rm -f chamber && make dtu

dsql:
	docker exec -it chamber psql -U postgres -h localhost -p 5432
