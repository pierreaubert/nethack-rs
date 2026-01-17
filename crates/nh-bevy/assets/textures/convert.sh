#!/bin/sh
for i in high-resolution/*.jpeg; do
    k=${i#high-resolution/}; magick "$i" -resize 256x256 ${k};
done
