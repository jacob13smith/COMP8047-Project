import 'package:flutter/material.dart';
import 'dart:io';
import 'dart:convert';

void main() {
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Blockchain EHR System',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
        useMaterial3: true,
      ),
      home: const MyHomePage(title: 'Home'),
    );
  }
}

class MyHomePage extends StatefulWidget {
  const MyHomePage({super.key, required this.title});

  final String title;

  @override
  State<MyHomePage> createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  InternetAddress? host;

  void _connectToDaemon() {
    host = InternetAddress("/tmp/ehr.sock", type: InternetAddressType.unix);
    RawSocket.connect(host, 0).then((socket) {
      socket.listen((event) {
        if (event == RawSocketEvent.read) {
          // Data is available to read
          List<int> data = socket.read() as List<int>;
          if (data != null) {
            String message = String.fromCharCodes(data);
            var json = jsonDecode(message);
            print('Received data: $json');
          }
        } else if (event == RawSocketEvent.write) {
          // The socket is ready for writing
          print('Socket is ready for writing.');
        } else if (event == RawSocketEvent.readClosed) {
          // The remote end has closed the connection
          print('Connection closed by the remote end.');
          socket.close();
        } else if (event == RawSocketEvent.closed) {
          // The socket has been fully closed
          print('Socket is fully closed.');
        }
      });
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
        title: Text(widget.title),
      ),
      body: const Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: <Widget>[],
        ),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: _connectToDaemon,
        tooltip: 'Connect to daemon',
        child: const Icon(Icons.add),
      ),
    );
  }
}
