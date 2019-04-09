#!/bin/bash

set -exu

ORDER=(ethabi derive contract cli)

for crate in ${ORDER[@]}; do
	cd $crate
	cargo publish $@
	cd -
done

