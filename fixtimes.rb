#!/usr/bin/ruby -w

require "fileutils"
require "json"

db = JSON.load(ARGF)

db.each do | book |
  next unless book["filename"]
  next unless book["info"]
  next unless book["info"]["updated"]
  updated = Time.at(book["info"]["updated"])
  path = "books/#{book["filename"]}"
  next unless File.exist?(path)
  $stdout.write("#{path}\n")
  FileUtils.touch(path, mtime: updated)
end
