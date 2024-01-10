import 'dart:typed_data';

import 'socket_api.dart';

import 'package:flutter/material.dart';
import 'dart:io';
import 'dart:convert';

void main() {
  runApp(MyApp());
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
      home: const WelcomePage(title: 'Home'),
    );
  }
}

class WelcomePage extends StatefulWidget {
  const WelcomePage({super.key, required this.title});

  final String title;

  @override
  State<WelcomePage> createState() => _WelcomePageState();
}

class _WelcomePageState extends State<WelcomePage> {
  late SocketApi socketApi;
  Future<void> connect() async {
    socketApi = SocketApi('/tmp/ehr_0.sock', '/tmp/ehr_1.sock');
    try {
      await socketApi.connect();

      socketApi.sendMessage('Hello from Flutter!');

      // Add more logic here as needed
    } catch (e) {
      print('Error: $e');
    } finally {
      //socketApi.close();
    }
  }

  void createNewBlockchain() {
    socketApi.sendMessage("Create new chain");
  }

  @override
  Widget build(BuildContext context) {
    connect();
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
        onPressed: createNewBlockchain,
        tooltip: 'New Blockchain',
        child: const Icon(Icons.add),
      ),
    );
  }
}
