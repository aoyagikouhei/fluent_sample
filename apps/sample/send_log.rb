require "fluent-logger"
require "socket"

# Wait for fluentd to be ready
10.times do |i|
  begin
    sock = TCPSocket.new("fluentd", 24224)
    sock.close
    puts "fluentd is ready!"
    break
  rescue Errno::ECONNREFUSED
    puts "Waiting for fluentd... (#{i + 1}/10)"
    sleep 2
  end
end

logger = Fluent::Logger::FluentLogger.new(nil, host: "fluentd", port: 24224)

puts "Sending log to fluentd via forward protocol..."

result = logger.post("log.access", { "message" => "hello from ruby fluent-logger" })

if result
  puts "Log sent successfully!"
else
  puts "Failed to send log: #{logger.last_error}"
end

logger.close
