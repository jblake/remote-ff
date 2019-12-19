#!/usr/bin/ruby -w

require "json"

db = JSON.load(ARGF)

db.each do | book |
  $stdout.write("UPDATE books SET addTime = '#{book["info"]["updated"]}000' WHERE filename LIKE '%/#{book["filename"]}';\n")
end
