#!/bin/sh

mkdir -p sample.d

which usql || exec sh -c 'echo usql missing.; exit 1'

export ENV_ZIP_FILENAME=./sample.d/output.zip

export PGHOST=127.0.0.1
export PGUSER=postgres
export PGDATABASE=postgres
export PGPASSWORD="${PGPASSWORD}"

usql "pg://${PGUSER}@${PGHOST}" -c "
	CREATE TABLE IF NOT EXISTS tab1(
		id BIGSERIAL PRIMARY KEY,
		key TEXT NOT NULL,
		val TEXT NOT NULL
	)
"

usql "pg://${PGUSER}@${PGHOST}" -c "
	CREATE TABLE IF NOT EXISTS tab2(
		id BIGSERIAL PRIMARY KEY,
		key TEXT NOT NULL,
		val TEXT NOT NULL
	)
"

printf \
	'%s\n' \
	'tab1' \
	'tab2' |
	./rs-tables2pgcopy2zip

unzip -lv "${ENV_ZIP_FILENAME}"
