#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
: "${CLOUDFRONT_DISTRIBUTION_ID:?Set the Polish CloudFront distribution ID}"
python3 scripts/verify-website.py
node --check website/water.js
node scripts/test-website-demo.mjs
aws s3 sync website/ s3://polish.salsaparapizza.com/ --cache-control 'public,max-age=300' --only-show-errors
aws s3 cp website/index.html s3://polish.salsaparapizza.com/index.html --content-type 'text/html; charset=utf-8' --cache-control 'no-cache,max-age=0,must-revalidate' --only-show-errors
invalidation=$(aws cloudfront create-invalidation --distribution-id "$CLOUDFRONT_DISTRIBUTION_ID" --paths '/*' --query Invalidation.Id --output text)
aws cloudfront wait invalidation-completed --distribution-id "$CLOUDFRONT_DISTRIBUTION_ID" --id "$invalidation"
printf 'Published Polish website; invalidation %s completed.\n' "$invalidation"
