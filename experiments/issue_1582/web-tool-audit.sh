#!/usr/bin/env bash
set -eu
exec > /tmp/issue-1582-web-tool-audit.log 2>&1
rg -n 'web_search_preview|web_search.*type|web_search_call|web_search_tool' js/server rust/src || test "$?" -eq 1
