# Project makefile
#
# I'm unable to get docker to build my project with a single command,
# so it's makefile time.

image_name = ntu_canoebot
manifest = $(shell find -name "Cargo.*" -type f)

default: build

cache:
	docker build -t ${image_name}_cache -f docker/cache.Dockerfile .

build: cache
	docker compose build

up:
	docker compose up -d

buildup: build
	docker compose up -d

down:
	docker compose down

save:
	docker save --output $(image_name).tar $(image_name)

info:
	$(info $$manifest is [${manifest}])

logs:
	docker compose logs -f --tail 10
