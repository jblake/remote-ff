#!/usr/bin/ruby -w

require "faraday"
require "json"
require "nokogiri"
require "time"

CHECK_PROBABILITY = 0.25 # Will check for updated metadata with this probability
FORCE_PROBABILITY = 0.05 # Will download regardless of metadata with this probability

FFN_CHAPTER = /chapter\s+\S+:?\s+(.+?)\s*$/i

def escape(text)
  return Nokogiri::XML::Text.new(text, Nokogiri::XML::Document.new()).to_s()
end

def enterp(fh, formatting)
  fh.write("<p>")
  formatting.each() do | n |
    fh.write("<#{n}>")
  end
end

def leavep(fh, formatting)
  formatting.reverse().each() do | n |
    fh.write("</#{n}>")
  end
  fh.write("</p>\n")
end

def newp(fh, inp, formatting)
  leavep(fh, formatting) if inp
  enterp(fh, formatting)
end

def sanitize(parent, fh, inp, formatting)
  was_inp = inp
  parent.children.each() do | node |
    if node.text? then
      txt = node.to_s()
      blank = txt.strip() == ""
      enterp(fh, formatting) unless blank or inp
      fh.write(txt) unless blank and not inp
      inp = true unless blank
    elsif node.element? then
      case node.name
      when "a", "img"
        sanitize(node, fh, inp, formatting)
      when "center", "p", "blockquote", "li"
        newp(fh, inp, formatting)
        sanitize(node, fh, true, formatting)
        inp = true
      when "hr"
        leavep(fh, formatting) if inp
        fh.write("<empty-line/>")
        enterp(fh, formatting) if inp
      when "br"
        newp(fh, inp, formatting)
        inp = true
      when "b", "strong"
        fh.write("<strong>") if inp
        sanitize(node, fh, inp, formatting + ["strong"])
        fh.write("</strong>") if inp
      when "i", "em"
        fh.write("<emphasis>") if inp
        sanitize(node, fh, inp, formatting + ["emphasis"])
        fh.write("</emphasis>") if inp
      when "span", "u"
        fh.write("<strong><emphasis>") if inp
        sanitize(node, fh, inp, formatting + ["strong", "emphasis"])
        fh.write("</emphasis></strong>") if inp
      when "ol", "ul"
        sanitize(node, fh, inp, formatting)
      else
        throw "Unrecognized tag #{node.name}"
      end
    end
  end
  enterp(fh, formatting) if was_inp and not inp
  leavep(fh, formatting) if inp and not was_inp
end

def mkfb2(path, info, parts)
  File.open("#{path}.tmp", "w") do | fh |
    fh.write(<<END)
<?xml version="1.0" encoding="UTF-8"?>
<FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0">
  <description>
    <title-info>
      <author><nickname>#{escape(info["author"])}</nickname></author>
      <book-title>#{escape(info["title"])}</book-title>
      <date>#{escape(Time.at(info["updated"]).utc().strftime("%Y-%m-%d %H:%M:%S UTC"))}</date>
    </title-info>
    <document-info>
      <author><nickname>jblake</nickname></author>
      <date>#{escape(Time.now().utc().strftime("%Y-%m-%d %H:%M:%S UTC"))}</date>
      <program-used>remote-ff 4</program-used>
    </document-info>
  </description>
  <body>
END

    parts.each() do | part |
      fh.write(<<END)
    <section>
      <title><p>#{escape(part["title"])}</p></title>
END
      sanitize(part["content"], fh, false, [])
      fh.write(<<END)
    </section>
END
    end

    fh.write(<<END)
  </body>
</FictionBook>
END
  end
  File.rename("#{path}.tmp", path)
end

def safepath(path)
  return path.gsub(/[^a-zA-Z0-9]+/, '_').gsub(/^_/, '').gsub(/_$/, '')
end

$db = JSON.load(File.open("db.json"))

def writedb()
  JSON.dump($db, File.open("db.json.tmp", "w"))
  File.rename("db.json.tmp", "db.json")
end

($db.size() - 1).downto(0) do | i |
  begin
    entry = $db[i]
    p = rand()

    if entry["missing"] == true then
      entry["missing"] = 10
      writedb()
    elsif entry["missing"] == false then
      entry["missing"] = 0
      writedb()
    end

    next unless p < CHECK_PROBABILITY or p < FORCE_PROBABILITY

    next if entry["pruned"]
    next if entry["missing"] and entry["missing"] >= 10

    case entry["site"]

    when "ffn"
      html = Faraday.get("https://fanfiction.jblake.org/s/#{entry["id"]}").body
      page = Nokogiri::HTML.parse(html)
      title = page.xpath("//div[@id=\"content\"]/div[1]/b/text()")[0].to_s().strip()
      author = page.xpath("//div[@id=\"content\"]/div[1]/a/text()")[0].to_s().strip()
      updated = page.xpath("//div[@id=\"content\"]//@data-xutime").map { | x | x.value.to_i }.max()
      chapters = page.xpath("//hr/following-sibling::div/a/text()")
      if chapters.size() >= 1 and chapters[0].to_s() =~ /^(\d+)$/ then
        chapters = $1.to_i()
      else
        chapters = 1
      end
      if title == "" then
        $stdout.write("#{i}: <missing>, entry: #{entry.inspect}\n")
        entry["missing"] = 0 unless entry["missing"]
        entry["missing"] += 1
        writedb()
        next
      end
      $stdout.write("#{i}: #{title} by #{author}\n")

      if entry["missing"] and entry["missing"] > 0 then
        entry.delete("missing")
        writedb()
      end

      info = {
        "author" => author,
        "chapters" => chapters,
        "title" => title,
        "updated" => updated,
      }

      if not entry["filename"] or not File.exist?("books/#{entry["filename"]}") then
        $stdout.write("\tLocal file does not exist, will download.\n")
        entry["filename"] = "#{i}.ffn#{safepath(entry["id"])}.#{safepath(title)}.#{safepath(author)}.fb2" unless entry["filename"]
      elsif entry["info"] != info then
        $stdout.write("\tMetadata has been changed, will download.\n")
        $stdout.write("\t\told: #{entry["info"].inspect}\n")
        $stdout.write("\t\tnew: #{info.inspect}\n")
      elsif p < FORCE_PROBABILITY then
        $stdout.write("\tRandomly forced, will download.\n")
      else
        next
      end

      ctitle = page.xpath("//div[@id=\"content\"]/text()")[-1].to_s()
      if ctitle =~ FFN_CHAPTER then
        ctitle = "Chapter 1: #{$1}"
      else
        ctitle = "Chapter 1"
      end

      ccontent = page.xpath("//div[@id=\"storycontent\"]")[0]

      parts = [{"title" => ctitle, "content" => ccontent}]

      2.upto(chapters) do | c |
#        $stdout.write("\tFetching chapter #{c}...\n")
        chtml = Faraday.get("https://fanfiction.jblake.org/s/#{entry["id"]}/#{c}").body
        cpage = Nokogiri::HTML.parse(chtml)
        title = cpage.xpath("//div[@id=\"content\"]/div[1]/b/text()")[0].to_s()
        if title == "" then
          $stdout.write("\tCouldn't fetch chapter #{c}, giving up.\n")
          throw "Couldn't fetch chapter in download phase"
        end

        ctitle = cpage.xpath("//div[@id=\"content\"]/text()")[-1].to_s()
        if ctitle =~ FFN_CHAPTER then
          ctitle = "Chapter #{c}: #{$1}"
        else
          ctitle = "Chapter #{c}"
        end

        ccontent = cpage.xpath("//div[@id=\"storycontent\"]")[0]

        parts << {"title" => ctitle, "content" => ccontent}
      end

#      $stdout.write("\tSanitizing and writing...\n")
      mkfb2("books/#{entry["filename"]}", info, parts)
      entry["info"] = info
      writedb()

    when "hpffa"
      html = Faraday.get("http://www.hpfanficarchive.com/stories/viewstory.php?sid=#{entry["id"]}").body.force_encoding("iso-8859-1").encode("utf-8")
      page = Nokogiri::HTML.parse(html)
      title = page.xpath("//div[@id=\"pagetitle\"]/a[1]/text()")[0].to_s().strip()
      author = page.xpath("//div[@id=\"pagetitle\"]/a[2]/text()")[0].to_s().strip()
      chapters = page.xpath("//span[@class=\"label\" and .=\"Chapters: \"]/following-sibling::text()[1]")[0].to_s().strip().to_i()
      updated = page.xpath("//comment()[.=\" PUBLISHED START \"]/following-sibling::text()[1]")
      updated += page.xpath("//comment()[.=\" UPDATED START \"]/following-sibling::text()[1]")
      updated = updated.map { | x | Time.parse(x.to_s()).utc().to_i() }.max()
      if title == "" then
        $stdout.write("#{i}: <missing>, entry: #{entry.inspect}\n")
        entry["missing"] = 0 unless entry["missing"]
        entry["missing"] += 1
        writedb()
        next
      end
      $stdout.write("#{i}: #{title} by #{author}\n")

      if entry["missing"] and entry["missing"] > 0 then
        entry.delete("missing")
        writedb()
      end

      info = {
        "author" => author,
        "chapters" => chapters,
        "title" => title,
        "updated" => updated,
      }

      if not entry["filename"] or not File.exist?("books/#{entry["filename"]}") then
        $stdout.write("\tLocal file does not exist, will download.\n")
        entry["filename"] = "#{i}.hpffa#{safepath(entry["id"])}.#{safepath(title)}.#{safepath(author)}.fb2" unless entry["filename"]
      elsif entry["info"] != info then
        $stdout.write("\tMetadata has been changed, will download.\n")
        $stdout.write("\t\told: #{entry["info"].inspect}\n")
        $stdout.write("\t\tnew: #{info.inspect}\n")
      elsif p < FORCE_PROBABILITY then
        $stdout.write("\tRandomly forced, will download.\n")
      else
        next
      end

      parts = []

      1.upto(chapters) do | c |
#        $stdout.write("\tFetching chapter #{c}...\n")
        chtml = Faraday.get("http://www.hpfanficarchive.com/stories/viewstory.php?action=printable&sid=#{entry["id"]}&chapter=#{c}").body.force_encoding("iso-8859-1").encode("utf-8")
        cpage = Nokogiri::HTML.parse(chtml)
        title = cpage.xpath("//div[@id=\"pagetitle\"]/a[1]/text()")[0].to_s().strip()
        if title == "" then
          $stdout.write("\tCouldn't fetch chapter #{c}, giving up.\n")
          throw "Couldn't fetch chapter in download phase"
        end

        ctitle = cpage.xpath("//div[@class=\"chaptertitle\"]/text()")[0].to_s().strip()
        if ctitle =~ /^(.+?) by .+?$/ then
          ctitle = $1.strip()
        end

        ccontent = cpage.xpath("//div[@class=\"chapter\"]")[0]

        parts << {"title" => ctitle, "content" => ccontent}
      end

#      $stdout.write("\tSanitizing and writing...\n")
      mkfb2("books/#{entry["filename"]}", info, parts)
      entry["info"] = info
      writedb()

    else
      $stdout.write("#{JSON.pretty_generate(entry)}\n")
      throw "Unrecognized site"
    end

  rescue Interrupt => e
    throw e
  rescue SignalException => e
    throw e
  rescue Object => e
    $stdout.write("! Got an exception in entry #{i}: #{e}\n")
  end
end
