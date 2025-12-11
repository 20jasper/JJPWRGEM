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
  hyperfine --warmup 3 --sort mean-time \
    --export-markdown "$pretty_md" \
    --export-json "$pretty_json" \
    --command-name "jjp" \
      "jjp format < $FILE_PATH" \
    --command-name "jello" \
      "jello < $FILE_PATH" \
    --command-name "prettier" \
      "prettier --parser json < $FILE_PATH" \
    --command-name "jq" \
      "jq < $FILE_PATH" \
    --command-name "gojq" \
      "gojq < $FILE_PATH" \
    --command-name "jaq" \
      "jaq < $FILE_PATH" \
    --command-name "sjq" \
      "sjq . --pretty < $FILE_PATH" \
    --command-name "dprint" \
      "dprint fmt --stdin .json < $FILE_PATH" \
    --command-name "python" \
      "python -m json.tool < $FILE_PATH" \
    --command-name "node" \
      "node -e '(async()=>{const t=await require(\"node:stream/consumers\").text(process.stdin);process.stdout.write(JSON.stringify(JSON.parse(t),null,2))})()' < $FILE_PATH" \
    --command-name "bun" \
      "bun -e 'console.log(JSON.stringify(await new Response(Bun.stdin.stream()).json(), null, 2))' < $FILE_PATH" \
    --command-name "jshon" \
      "jshon < $FILE_PATH" \
    --command-name "jsonxf" \
      "jsonxf < $FILE_PATH" \
    --command-name "jsonformat" \
      "jsonformat < $FILE_PATH" 

  # uglify
  hyperfine --warmup 3 --sort mean-time \
    --export-markdown "$ugly_md" \
    --export-json "$ugly_json" \
    --command-name "jjp" \
      "jjp format -u < $FILE_PATH" \
    --command-name "json-minify" \
      "json-minify < $FILE_PATH" \
    --command-name "jello" \
      "jello -c < $FILE_PATH" \
    --command-name "jsonxf" \
      "jsonxf -m < $FILE_PATH" \
    --command-name "jq" \
      "jq -c < $FILE_PATH" \
    --command-name "jaq" \
      "jaq -c < $FILE_PATH" \
    --command-name "gojq" \
      "gojq -c < $FILE_PATH" \
    --command-name "sjq" \
      "sjq . < $FILE_PATH" \
    --command-name "python" \
      "python -m json.tool --compact < $FILE_PATH" \
    --command-name "minify" \
      "minify --type json < $FILE_PATH" \
    --command-name "node" \
      "node -e '(async()=>{const t=await require(\"node:stream/consumers\").text(process.stdin);process.stdout.write(JSON.stringify(JSON.parse(t)))})()' < $FILE_PATH" \
    --command-name "bun" \
      "bun -e 'console.log(JSON.stringify(await new Response(Bun.stdin.stream()).json()))' < $FILE_PATH"
}

for file in "${FILES[@]}"
do 
  full_path="./data/$file.json"
  bench "$full_path" "$file"
done



