SHELL:=/bin/bash

# .DEFAULT_GOAL := default
.PHONY: check fix format lint dev release publish clean

# "This will essentially compile the packages without performing the final step of code generation, which is faster than running cargo build."
check:
	cargo check

fix:
	cargo fix --allow-staged

format:
	cargo fmt

lint:
	cargo clippy
	-cargo audit

dev:
	cargo run --features bevy/dynamic

web-dev:
	wasm-pack build --target web --dev

web-release:
	wasm-pack build --target web --release

release: lint
	cargo run --release

publish: web-release
	@echo "====> deploying to github"
	# checkout the existing gh-pages
	rm -rf /tmp/gh-pages
	git worktree add -f /tmp/gh-pages gh-pages
	rm -rf /tmp/gh-pages/*
	# copy the web files to the gh-pages folder
	cp index.html /tmp/gh-pages/
	mkdir -p /tmp/gh-pages/pkg
	cp -rp pkg/*.js /tmp/gh-pages/pkg
	cp -rp pkg/*.wasm /tmp/gh-pages/pkg
	cp -rp assets /tmp/gh-pages/
	# push our new gh-pages
	cd /tmp/gh-pages && \
		git add -A && \
		git commit -m "deployed on $(shell date) by ${USER}" && \
		git push origin gh-pages
	git worktree remove /tmp/gh-pages

clean:
	cargo clean