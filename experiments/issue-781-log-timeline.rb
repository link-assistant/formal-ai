#!/usr/bin/env ruby
# frozen_string_literal: true

# Extract the user-visible assistant narration and tool actions from a verbose
# Claude Code / solve.mjs JSON log. The raw issue-781 comparison log is more
# than 67,000 lines; this produces a small, reviewable event timeline without
# discarding timestamps or tool arguments.

require "json"

abort "usage: #{$PROGRAM_NAME} LOG" unless ARGV.length == 1

events = []
buffer = nil

File.foreach(ARGV.fetch(0)) do |line|
  match = line.match(/^\[([^\]]+)\] \[INFO\] ?(.*)$/m)
  next unless match

  timestamp = match[1]
  payload = match[2]
  if buffer.nil?
    next unless payload == "{\n" || payload == "{"

    buffer = +payload
  else
    buffer << payload
  end

  next unless payload.strip == "}"

  begin
    object = JSON.parse(buffer)
  rescue JSON::ParserError
    next
  end
  buffer = nil

  next unless object["type"] == "assistant"

  object.dig("message", "content").to_a.each do |content|
    case content["type"]
    when "text"
      text = content["text"].to_s.gsub(/\s+/, " ").strip
      events << [timestamp, "narration", text] unless text.empty?
    when "tool_use"
      arguments = JSON.generate(content["input"] || {})
      events << [timestamp, "tool", "#{content["name"]} #{arguments}"]
    end
  end
end

events.each_with_index do |(timestamp, kind, detail), index|
  puts format("%04d\t%s\t%s\t%s", index + 1, timestamp, kind, detail)
end
