#!/bin/bash

set -exu

VERSION=$(grep "^version" ./ethabi/Cargo.toml | sed -e 's/.*"\(.*\)"/\1/')
ORDER=(ethabi derive contract cli)

for crate in ${ORDER[@]}; do
	cd $crate
	cargo publish $@
	cd -
done

echo "Tagging version $VERSION"
git tag -a v$VERSION -m "Version $VERSION"
git push --tags
