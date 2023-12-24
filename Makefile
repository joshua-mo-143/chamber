dshuttle:
	docker exec -it shuttle_boulder-server_shared_postgres psql -U postgres -h localhost -p 5432

dt:
	docker run -d -t -p 8500:5432 -e POSTGRES_PASSWORD=postgres --name boulder postgres

dtr:
	docker rm -f boulder && make docker

dsql:
	docker exec -it boulder psql -U postgres -h localhost -p 5432

