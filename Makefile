install:
	cargo add dotenv@0.15.0
	cargo add tokio@1.28.2 -F "full"
	cargo add base64@0.21.2
	cargo add sys-info@0.9.1
	cargo add chrono@0.4.24 -F "serde"
	cargo add serde@1.0.163 -F "derive"
	cargo add serde_json@1.0.96
	cargo add oracle@0.5.7 -F "chrono stmt_without_lifetime aq_unstable"
	cargo add sqlx@0.6.3 -F "runtime-async-std-native-tls mysql chrono"
	# Agent log0 hash.
	cargo add sha256@1.1.3
	cargo add sha1@0.10.5
	cargo add md5@0.7.0
	# Run same nodemon.
	cargo install cargo-watch@8.4.0

run:
	cargo-watch -q -c -w src/ -x run
