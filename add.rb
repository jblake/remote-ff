#!/usr/bin/ruby -w

require "fileutils"
require "json"

$db = JSON.load(File.open("db.json"))
$need_fetch = []

def writedb()
  JSON.dump($db, File.open("db.json.tmp", "w"))
  File.rename("db.json.tmp", "db.json")
end

def add(url)
  case url
  when /^https?:\/\/(?:www\.|m\.)?fanfiction\.net\/s\/(\d+)/
    site = "ffn"
    id = $1
  when /^https?:\/\/(?:www\.)?hpfanficarchive\.com\/stories\/viewstory\.php\?(?:[^&;]+[&;])*sid=(\d+)/
    site = "hpffa"
    id = $1
  else
    raise "Unrecognized URL #{url}"
  end
  $stdout.write("#{site} #{id} => ")
  $db.each_with_index() do | entry, index |
    if entry["site"] == site and entry["id"] == id then
      $stdout.write("#{index} (already existed)\n")
      return
    end
  end
  $stdout.write("#{$db.size()}\n")
  $need_fetch.push($db.size())
  $db.push({"site": site, "id": id})
end

ARGV.each() do | url |
  add(url)
end

if $need_fetch.empty?() then
  $stdout.write("No changes needed.\n")
else
  $stdout.write("Saving database...\n")
  writedb()
  $stdout.write("Need download: #{$need_fetch.join(" ")}\n")
end
