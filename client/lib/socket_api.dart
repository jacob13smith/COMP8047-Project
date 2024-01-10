import 'dart:io';
import 'dart:convert';

class SocketApi {
  late String readSocketPath;
  late String writeSocketPath;
  late RawSocket writeSocket;
  late RawSocket readSocket;

  SocketApi(this.readSocketPath, this.writeSocketPath);

  // Connect to the Unix socket
  Future<void> connect() async {
    InternetAddress? hostWrite =
        InternetAddress(writeSocketPath, type: InternetAddressType.unix);
    InternetAddress? hostRead =
        InternetAddress(readSocketPath, type: InternetAddressType.unix);

    writeSocket = await RawSocket.connect(hostWrite, 0);
    readSocket = await RawSocket.connect(hostRead, 0);
  }

  void readFromSocket(RawSocket socket) {
    // Data is available to read
    List<int> data = readSocket.read() as List<int>;
    if (data.isNotEmpty) {
      String message = String.fromCharCodes(data);
      var json = jsonDecode(message);
      print('Received data: $json');
    }
  }

  // Send a message to the Rust daemon
  void sendMessage(String message) {
    writeSocket.write(utf8.encode(message));
  }

  // Close the socket connection
  void close() {
    writeSocket.close();
    readSocket.close();
  }
}
