#!/usr/bin/env ruby

require 'socket'
PORT = 1234

def log(str)
  puts "[#{Time.now}] #{str}"
end

def handle_line(line, socket)
  case line.chomp(";")
  when "##CN"
    log "Received connection request, allowing connections"
    socket.write "##CN1;"
  when "##ID10705kenwoodadmin"
    log "Successfully authenticated user"
    client_authenticated = true
    socket.write "##ID1"
  when /^##ID/
    log "Incorrect authentication, rejecting: #{line}"
    socket.write "##ID0;"
  when "AI1"
    log "Auto-info ON"
    socket.write line
  when "PS"
    socket.write "PS1;"
  else
    log "Received unknown command: #{line.inspect}"
    socket.write "?;"
  end

  socket.flush()
end

log "starting tcp server on :#{PORT}"

TCPServer.open(PORT) do |server|
  log "**** Server waiting for connections"

  loop do
    socket = server.accept

    log "*** new client"
    log "Accepted connection on socket: #{socket.inspect}"

    client_authenticated = false

    while line = socket.gets(";") do
      log "socket.gets raw line: #{line.inspect}"

      if !line
        log "Read nil line, sleeping"
        sleep 2
        next
      end

      handle_line(line, socket)
    end

    log "End of socket read loop"
    socket.close
  end

  server.close
end

