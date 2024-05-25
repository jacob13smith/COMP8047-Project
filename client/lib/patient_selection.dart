import 'dart:async';
import 'dart:io';

import 'package:client/patient_page.dart';
import 'package:client/socket_api.dart';
import 'package:intl/intl.dart';
import 'package:flutter/material.dart';

class PatientSelectionPage extends StatefulWidget {
  const PatientSelectionPage({super.key, required this.title});
  final String title;

  @override
  State<PatientSelectionPage> createState() => _PatientSelectionPage();
}

class _PatientSelectionPage extends State<PatientSelectionPage> {
  late SocketApi socketApi;
  List chains = [];

  DateTime? _selectedDate;
  late TextEditingController _dateController;
  final TextEditingController _firstNameController = TextEditingController();
  final TextEditingController _lastNameController = TextEditingController();

  @override
  void initState() {
    super.initState();
    _dateController = TextEditingController();
    connect().then((_) => {
          requestChains(),
          Timer.periodic(const Duration(seconds: 3), (timer) {
            requestChains();
          })
        });
  }

  Future<void> connect() async {
    String? userHome =
        Platform.environment['HOME'] ?? Platform.environment['USERPROFILE'];
    socketApi = SocketApi('$userHome/.ehr/ehr.sock');
    try {
      await socketApi.connect();
    } catch (e) {
      print(e);
    }
  }

  void requestChains() async {
    Map<String, dynamic> jsonRequest = {
      'action': 'get_chains',
      'parameters': {}
    };
    List fetchedChains = await socketApi.sendRequest(jsonRequest);
    setState(() {
      chains = fetchedChains;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
        title: Text(widget.title),
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: <Widget>[
            SizedBox(
              height: 400,
              width: 300,
              child: ListView.builder(
                itemCount: chains.length,
                itemBuilder: (context, index) {
                  return ListTile(
                    title: Text(
                        '${chains[index]['last_name']}, ${chains[index]['first_name']}'),
                    onTap: () {
                      // Handle button tap (e.g., navigate to a new screen)
                      String title =
                          '${chains[index]['last_name']}, ${chains[index]['first_name']}';
                      Navigator.push(
                        context,
                        MaterialPageRoute(
                            builder: (context) => PatientPage(
                                title: title,
                                id: chains[index]['id'],
                                socketApi: socketApi)),
                      );
                    },
                  );
                },
              ),
            ),
            const SizedBox(height: 20),
            ElevatedButton(
              onPressed: () async {
                // Handle button tap
                showModalBottomSheet(
                  context: context,
                  builder: (BuildContext context) {
                    return SingleChildScrollView(
                      child: Container(
                        padding: const EdgeInsets.symmetric(
                            vertical: 10.0, horizontal: 20.0),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.stretch,
                          children: <Widget>[
                            const Text(
                              'Add Patient',
                              style: TextStyle(
                                fontSize: 24.0,
                                fontWeight: FontWeight.bold,
                              ),
                            ),
                            const SizedBox(height: 20.0),
                            // Add your form fields here
                            // Example TextField:
                            TextField(
                              controller: _firstNameController,
                              decoration: const InputDecoration(
                                labelText: 'First Name',
                                border: OutlineInputBorder(),
                              ),
                            ),
                            const SizedBox(height: 20.0),
                            TextField(
                              controller: _lastNameController,
                              decoration: const InputDecoration(
                                labelText: 'Last Name',
                                border: OutlineInputBorder(),
                              ),
                            ),
                            const SizedBox(height: 20.0),
                            // Date of Birth picker
                            TextFormField(
                              onTap: () async {
                                final DateTime? pickedDate =
                                    await showDatePicker(
                                  context: context,
                                  initialDate: _selectedDate ?? DateTime.now(),
                                  firstDate: DateTime(1900),
                                  lastDate: DateTime.now(),
                                );
                                if (pickedDate != null) {
                                  setState(() {
                                    _selectedDate = pickedDate;
                                    _dateController.text =
                                        DateFormat('yyyy-MM-dd')
                                            .format(_selectedDate!);
                                  });
                                }
                              },
                              readOnly: true, // Make the text field read-only
                              controller: _dateController,
                              decoration: const InputDecoration(
                                labelText: 'Date of Birth',
                                border: OutlineInputBorder(),
                              ),
                            ),
                            const SizedBox(height: 20.0),
                            ElevatedButton(
                              onPressed: () async {
                                String firstName = _firstNameController.text;
                                String lastName = _lastNameController.text;
                                String dateOfBirth = _dateController.text;

                                Map<String, dynamic> jsonRequest = {
                                  'action': 'create_chain',
                                  'parameters': {
                                    'first_name': firstName,
                                    'last_name': lastName,
                                    'date_of_birth': dateOfBirth
                                  }
                                };
                                await socketApi.sendRequest(jsonRequest).then(
                                    (response) => {Navigator.pop(context)});
                                requestChains();
                              },
                              child: const Text('Create Patient File'),
                            ),
                          ],
                        ),
                      ),
                    );
                  },
                );
              },
              child: const Text('Add Patient'),
            ),
          ],
        ),
      ),
    );
  }
}
