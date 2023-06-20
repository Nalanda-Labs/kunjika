# kunjika

This is a minimal forum/qa web app.
build with rust actix, diesel and postgres.

## running the app

### manually
Backend
```shell
cd rbackend
export DATABASE_URL='postgres://postgres:passwd@localhost'
cp template-sample.json template.json
cargo run
```

Frontend
```shell
cd ui
npm install
npm run dev -- --host localhost # not shure why this have to be here as i wont expect anything to change by this...
```

Database
```shell
docker run -d -p localhost:5432:5432 --name postgres postgres:15.3-alpine3.17
sleep 3
cat migrations/20220620050241_initial.sql | docker exec -it postgres psql -U postgres
```

Proxy with port redirection
```shell
docker run -d --name nginx-proxy-kunjika --network host -v ./docker/nginx/nginx.conf:/etc/nginx/nginx.conf:ro nginx
sudo socat TCP-LISTEN:80,fork TCP:127.0.0.1:8080
```

### docker-compose
see [docker-compose](docker-compose.yml) for more details
```shell
docker-compose up -d
```

### kubernetes
warning! experimental
```shell
kubectl apply -f https://raw.githubusercontent.com/Nalanda-Labs/kunjika/main/kubernetes.yml
```