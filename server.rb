require 'socket'
PORT = 1234

def handle_line(line, socket)
  case line.chomp(";")
  when "##CN"
    puts "Received connection request, allowing connections"
    socket.write "##CN1;"
  when "##ID10705kenwoodadmin"
    puts "Successfully authenticated user"
    client_authenticated = true
    socket.write "##ID1"
  when /^##ID/
    puts "Incorrect authentication, rejecting: #{line}"
    socket.write "##ID0;"
  when "AI1"
    puts "Auto-info ON"
    socket.write line
  when "PS"
    socket.write "PS1;"
  else
    puts "Received unknown command: #{line.inspect}"
    socket.write "?;"
  end

  socket.flush()
end

puts "starting tcp server on :#{PORT}"
TCPServer.open(PORT) do |server|
  puts "**** Server waiting for connections"

  loop do
    socket = server.accept

    puts "*** new client"
    puts "Accepted connection on socket: #{socket.inspect}"

    client_authenticated = false

    while line = socket.gets(";") do
      puts "socket.gets raw line: #{line.inspect}"

      if !line
        puts "Read nil line, sleeping"
        sleep 2
        next
      end

      handle_line(line, socket)
    end

    puts "End of socket read loop"
    socket.close
  end

  server.close
end

