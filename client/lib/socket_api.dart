import 'dart:async';
import 'dart:io';
import 'dart:convert';

class SocketApi {
  late String socketPath;
  late Socket socket;
  final Map<int, Completer<dynamic>> _responseCompleters = {};

  SocketApi(this.socketPath);

  // Connect to the Unix socket
  Future<void> connect() async {
    InternetAddress? host =
        InternetAddress(socketPath, type: InternetAddressType.unix);

    socket = await Socket.connect(host, 0);

    _startListening();
  }

  void _startListening() {
    socket.listen((List<int> data) {
      String response = String.fromCharCodes(data);
      _handleResponse(response);
    });
  }

  void _handleResponse(String response) {
    final responseJson = jsonDecode(response);
    int requestId = responseJson['id'];
    String responseData = responseJson['data'];

    if (_responseCompleters.containsKey(requestId)) {
      _responseCompleters[requestId]!.complete(jsonDecode(responseData));
      _responseCompleters.remove(requestId);
    }
  }

  // Send a message to the Rust daemon
  Future<dynamic> sendRequest(Map request) {
    int requestId = DateTime.now().millisecondsSinceEpoch;
    request['id'] = requestId;
    Completer<dynamic> completer = Completer<dynamic>();
    _responseCompleters[requestId] = completer;
    socket.write(jsonEncode(request));
    return completer.future;
  }

  // Close the socket connection
  void close() {
    socket.close();
  }
}
