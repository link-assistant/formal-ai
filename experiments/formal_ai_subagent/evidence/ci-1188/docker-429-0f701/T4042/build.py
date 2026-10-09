import pathlib,json,hashlib
p=pathlib.Path('/private/tmp/pr1188-ci-T4042');f=pathlib.Path('.github/workflows/container-images.yml');b=f.read_text();old='''        docker buildx build --load --provenance=false --platform "linux/$CONTAINER_ARCH" \\
         --file Dockerfile --build-arg BINARY_SOURCE=compile --tag formal-ai:independent-full \\
         --label "org.opencontainers.image.revision=$CONTAINER_SOURCE_COMMIT" --metadata-file standalone-build.json .''';assert b.count(old)==1
new='''        build_log=$(mktemp "$RUNNER_TEMP/formal-ai-independent-build.XXXXXX")
        trap 'rm -f "$build_log"' EXIT
        for build_attempt in 1 2 3; do
          set +e
'''+ '\n'.join('  '+line for line in old.splitlines())+''' 2>&1 | tee "$build_log"
          build_status=("${PIPESTATUS[@]}")
          set -e
          if [ "${build_status[1]}" -ne 0 ]; then exit "${build_status[1]}"; fi
          if [ "${build_status[0]}" -eq 0 ]; then break; fi
          if [ "$build_attempt" -eq 3 ] || ! grep -Eq '^ERROR: failed to build: failed to solve:.*unexpected status from HEAD request to https://registry-1\\.docker\\.io/v2/[^ ]+/manifests/[^ ]+: 429 Too Many Requests$' "$build_log"; then
            exit "${build_status[0]}"
          fi
          sleep "$((30 * build_attempt))"
        done'''
a=b.replace(old,new);changes=[dict(path=str(f.resolve()),before_sha256=hashlib.sha256(b.encode()).hexdigest(),after_sha256=hashlib.sha256(a.encode()).hexdigest(),content=a,before=b)];(p/'request.json').write_text(json.dumps(dict(changes=changes,original_job=114021525009),indent=2))
