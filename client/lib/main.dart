
import 'socket_api.dart';

import 'package:flutter/material.dart';

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
      home: const WelcomePage(title: 'Welcome to Blockchain EHR'),
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
  List chains = [];

  Future<void> connect() async {
    socketApi = SocketApi('/tmp/ehr.sock');
    try {
      await socketApi.connect();
    } catch (e) {}
  }

  void requestChains() async {
    Map<String, dynamic> jsonRequest = {'action': 'get_chains'};
    List response = await socketApi.sendRequest(jsonRequest);
    setState(() {
      chains = response;
    });
  }

  @override
  Widget build(BuildContext context) {
    connect().then((_) => requestChains());

    List<Widget> columnChildren = [];

    if (chains.isNotEmpty) {
      // Render based on the response data
      columnChildren.add(Text('Response: $chains'));
      // Add more widgets as needed
    }

    return Scaffold(
      appBar: AppBar(
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
        title: Text(widget.title),
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: <Widget>[
            if (chains.isNotEmpty)
              Expanded(
                child: ListView.builder(
                  itemCount: chains.length,
                  itemBuilder: (context, index) {
                    return ListTile(
                      title: Text(chains[index]['name']),
                      onTap: () {
                        // Handle button tap (e.g., navigate to a new screen)
                        print('Button tapped: ${chains[index]['name']}');
                      },
                    );
                  },
                ),
              ),
          ],
        ),
      ),
    );
  }
}
