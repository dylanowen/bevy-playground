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

web:
	wasm-pack build --target web --release

release: lint
	cargo run --release

publish: web
	@echo "====> deploying to github"
	# checkout the existing gh-pages
	rm -rf /tmp/gh-pages
	git worktree add -f /tmp/gh-pages gh-pages
	rm -rf /tmp/gh-pages/*
	# copy the web files to the gh-pages folder
	cp index.html /tmp/book/
	cp -rp pkg /tmp/book/
	cp -rp assets /tmp/book/

clean:
	cargo clean