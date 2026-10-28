#!/bin/bash

OUTPUT_DIR=${OUTPUT_DIR:-xtask/bench/output}
mkdir -p "$OUTPUT_DIR"

FILES=(
  "canada"
  "citm_catalog"
  "twitter"
)

bench() {
  FILE_PATH=$1
  FILE_NAME=$2

  echo "benchmarking $FILE_NAME: path $FILE_PATH"

  if [[ ! -f "$FILE_PATH" ]]; then
    echo "file not found: $FILE_PATH" 1>&2
    exit 1
  fi

  local pretty_md="$OUTPUT_DIR/pretty-$FILE_NAME.md"
  local pretty_json="$OUTPUT_DIR/pretty-$FILE_NAME.json"
  local ugly_md="$OUTPUT_DIR/ugly-$FILE_NAME.md"
  local ugly_json="$OUTPUT_DIR/ugly-$FILE_NAME.json"

  rm -f "$pretty_md" "$pretty_json" "$ugly_md" "$ugly_json"

  # pretty print
  hyperfine --input "$FILE_PATH" --warmup 3 \
    --export-markdown "$pretty_md" \
    --export-json "$pretty_json" \
    --command-name "jjp" \
      "jjp format" \
    --command-name "json-pp-rust" \
      "json-pp-rust" \
    --command-name "jsonice" \
      "jsonice" \
    --command-name "prettier" \
      "prettier --parser json" \
    --command-name "jq" \
      "jq" \
    --command-name "gojq" \
      "gojq" \
    --command-name "jaq" \
      "jaq" \
    --command-name "sjq" \
      "sjq . --pretty" \
    --command-name "dprint" \
      "dprint fmt --stdin .json" \
    --command-name "python" \
      "python -m json.tool" \
    --command-name "node" \
      "node -e '(async()=>{const t=await require(\"node:stream/consumers\").text(process.stdin);process.stdout.write(JSON.stringify(JSON.parse(t),null,2))})()'" \
    --command-name "bun" \
      "bun -e 'console.log(JSON.stringify(await new Response(Bun.stdin.stream()).json(), null, 2))'" \
    --command-name "jsonxf" \
      "jsonxf" \
    --command-name "jsonformat" \
      "jsonformat" \
    --command-name "oxfmt" \
      "oxfmt --stdin-filepath foo.json"

  # uglify
  hyperfine --input "$FILE_PATH" --warmup 3 \
    --export-markdown "$ugly_md" \
    --export-json "$ugly_json" \
    --command-name "jjp" \
      "jjp format -u" \
    --command-name "json-minify" \
      "json-minify" \
    --command-name "jsonxf" \
      "jsonxf -m" \
    --command-name "jq" \
      "jq -c" \
    --command-name "jaq" \
      "jaq -c" \
    --command-name "gojq" \
      "gojq -c" \
    --command-name "sjq" \
      "sjq ." \
    --command-name "python" \
      "python -m json.tool --compact" \
    --command-name "minify" \
      "minify --type json" \
    --command-name "node" \
      "node -e '(async()=>{const t=await require(\"node:stream/consumers\").text(process.stdin);process.stdout.write(JSON.stringify(JSON.parse(t)))})()'" \
    --command-name "bun" \
      "bun -e 'console.log(JSON.stringify(await new Response(Bun.stdin.stream()).json()))'"
}

for file in "${FILES[@]}"
do 
  full_path="./data/$file.json"
  bench "$full_path" "$file"
done



