#!/bin/bash
# Script to find all broken internal links in markdown files

find docs -name "*.md" -type f | while read file; do
    echo "Checking: $file"
    # Extract markdown links [text](path)
    grep -oP '\[.*?\]\(\K[^)]+' "$file" 2>/dev/null | while read link; do
        # Skip external links
        if [[ $link == http* ]] || [[ $link == mailto:* ]] || [[ $link == \#* ]]; then
            continue
        fi
        
        # Resolve relative path
        dir=$(dirname "$file")
        if [[ $link == /* ]]; then
            target="$link"
        else
            target="$dir/$link"
        fi
        
        # Check if file exists
        if [[ ! -e "$target" ]] && [[ ! -e "${target%.md}" ]]; then
            echo "  BROKEN: $link (in $file)"
        fi
    done
done
