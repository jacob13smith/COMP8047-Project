import 'package:client/socket_api.dart';
import 'package:flutter/material.dart';

class PatientPage extends StatefulWidget {
  const PatientPage(
      {super.key,
      required this.title,
      required this.id,
      required this.socketApi});

  final String title;
  final String id;
  final SocketApi socketApi;

  @override
  State<PatientPage> createState() => _PatientPage();
}

class _PatientPage extends State<PatientPage> {
  Map info = {};
  void requestPatientInfo() async {
    Map<String, dynamic> jsonRequest = {
      'action': 'get_patient_info',
      'parameters': {'id': widget.id}
    };
    dynamic patientInfo = await widget.socketApi.sendRequest(jsonRequest);

    print("Patient info: $patientInfo");
    setState(() {
      info = patientInfo;
    });
  }

  @override
  void initState() {
    super.initState();
    requestPatientInfo();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
        appBar: AppBar(),
        body: Container(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              // Section 1: Basic patient information and Providers
              Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  // Left container for basic patient information
                  Container(
                    width: MediaQuery.of(context).size.width *
                        0.4, // Adjust width as needed
                    decoration: BoxDecoration(
                      border: Border.all(color: Colors.black), // Add border
                    ),
                    padding: const EdgeInsets.all(16.0),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(widget.title,
                            style: const TextStyle(
                                fontWeight: FontWeight.bold, fontSize: 24)),
                        if (info.containsKey('date_of_birth'))
                          Text('Date of Birth: ${info['date_of_birth']}',
                              style:
                                  const TextStyle(fontWeight: FontWeight.bold)
                              // Add any desired style here
                              ),
                        // Add basic patient information here
                      ],
                    ),
                  ),
                  const SizedBox(width: 16.0), // Spacer between containers
                  // Right container for list of providers
                  Expanded(
                      child: Container(
                    height: 300,
                    decoration: BoxDecoration(
                      border: Border.all(color: Colors.black), // Add border
                    ),
                    padding: const EdgeInsets.all(16.0),
                    child: const Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text('Providers',
                            style: TextStyle(
                                fontWeight: FontWeight.bold, fontSize: 24)),
                        // Add list of providers here
                      ],
                    ),
                  )),
                ],
              ),
              // Section 2: List of record entries
              const SizedBox(height: 16),
              Expanded(
                child: Container(
                  padding: const EdgeInsets.all(16.0),
                  decoration: BoxDecoration(
                    border: Border.all(color: Colors.black), // Add border
                  ),
                  child: const Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text('Records',
                          style: TextStyle(
                              fontWeight: FontWeight.bold, fontSize: 24)),
                      // Add list of record entries here
                    ],
                  ),
                ),
              ),
            ],
          ),
        ));
  }
}
