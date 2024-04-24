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
  Map info = {"date_of_birth": "", "providers": [], "records": []};

  final TextEditingController _providerNameController = TextEditingController();
  final TextEditingController _providerIPController = TextEditingController();

  void requestPatientInfo() async {
    Map<String, dynamic> jsonRequest = {
      'action': 'get_patient_info',
      'parameters': {'id': widget.id}
    };
    dynamic patientInfo = await widget.socketApi.sendRequest(jsonRequest);

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
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: [
                        Row(
                          mainAxisAlignment: MainAxisAlignment.spaceBetween,
                          children: [
                            const Text('Providers',
                                style: TextStyle(
                                    fontWeight: FontWeight.bold, fontSize: 24)),
                            IconButton(
                              icon: const Icon(Icons.add),
                              onPressed: () {
                                showModalBottomSheet(
                                    context: context,
                                    builder: (BuildContext context) {
                                      return SingleChildScrollView(
                                          child: Container(
                                              padding:
                                                  const EdgeInsets.symmetric(
                                                      vertical: 10.0,
                                                      horizontal: 20.0),
                                              child: Column(children: [
                                                const Text(
                                                  'Add Provider',
                                                  style: TextStyle(
                                                    fontSize: 24.0,
                                                    fontWeight: FontWeight.bold,
                                                  ),
                                                ),
                                                const SizedBox(height: 20.0),
                                                TextField(
                                                  controller:
                                                      _providerNameController,
                                                  decoration:
                                                      const InputDecoration(
                                                    labelText: 'Name',
                                                    border:
                                                        OutlineInputBorder(),
                                                  ),
                                                ),
                                                const SizedBox(height: 20.0),
                                                TextField(
                                                  controller:
                                                      _providerIPController,
                                                  decoration:
                                                      const InputDecoration(
                                                    labelText: 'IP Address',
                                                    border:
                                                        OutlineInputBorder(),
                                                  ),
                                                ),
                                                const SizedBox(height: 20.0),
                                                ElevatedButton(
                                                  onPressed: () async {
                                                    String providerName =
                                                        _providerNameController
                                                            .text;
                                                    String providerIp =
                                                        _providerIPController
                                                            .text;

                                                    Map<String, dynamic>
                                                        jsonRequest = {
                                                      'action': 'add_provider',
                                                      'parameters': {
                                                        'chain_id': widget.id,
                                                        'name': providerName,
                                                        'ip': providerIp,
                                                      }
                                                    };
                                                    await widget.socketApi
                                                        .sendRequest(
                                                            jsonRequest)
                                                        .then((response) => {
                                                              Navigator.pop(
                                                                  context),
                                                              requestPatientInfo(),
                                                              _providerNameController
                                                                  .clear(),
                                                              _providerIPController
                                                                  .clear(),
                                                            });
                                                  },
                                                  child: const Text(
                                                      'Add Provider'),
                                                )
                                              ])));
                                    });
                                // Open modal for adding provider
                              },
                            )
                          ],
                        ),
                        DataTable(
                          border: TableBorder.all(),
                          columns: const [
                            DataColumn(label: Text('Name')),
                            DataColumn(label: Text('IP Address')),
                          ],
                          rows:
                              (info['providers'] as List<dynamic>).map((data) {
                            return DataRow(cells: [
                              DataCell(Text(data[0] ?? '')),
                              DataCell(Text(data[1] ?? '')),
                            ]);
                          }).toList(),
                        )
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
                    child: Column(
                        crossAxisAlignment: CrossAxisAlignment.stretch,
                        children: [
                          Row(
                            mainAxisAlignment: MainAxisAlignment.spaceBetween,
                            children: [
                              const Text('Records',
                                  style: TextStyle(
                                      fontWeight: FontWeight.bold,
                                      fontSize: 24)),
                              IconButton(
                                  icon: const Icon(Icons.add),
                                  onPressed: () {
                                    showModalBottomSheet(
                                        context: context,
                                        builder: (BuildContext context) {
                                          return SingleChildScrollView(
                                              child: Container(
                                                  padding: const EdgeInsets
                                                      .symmetric(
                                                      vertical: 10.0,
                                                      horizontal: 20.0),
                                                  child: Column(children: [
                                                    const Text(
                                                      'Add Record',
                                                      style: TextStyle(
                                                        fontSize: 24.0,
                                                        fontWeight:
                                                            FontWeight.bold,
                                                      ),
                                                    ),
                                                    const SizedBox(
                                                        height: 20.0),
                                                    TextField(
                                                      controller:
                                                          _providerNameController,
                                                      decoration:
                                                          const InputDecoration(
                                                        labelText: 'Subject',
                                                        border:
                                                            OutlineInputBorder(),
                                                      ),
                                                    ),
                                                    const SizedBox(
                                                        height: 20.0),
                                                    TextField(
                                                      controller:
                                                          _providerIPController,
                                                      decoration:
                                                          const InputDecoration(
                                                        labelText: 'Notes',
                                                        border:
                                                            OutlineInputBorder(),
                                                      ),
                                                    ),
                                                    const SizedBox(
                                                        height: 20.0),
                                                    ElevatedButton(
                                                      onPressed: () async {
                                                        Map<String, dynamic>
                                                            jsonRequest = {
                                                          'action':
                                                              'add_record',
                                                          'parameters': {
                                                            'chain_id':
                                                                widget.id,
                                                            'text': "text here"
                                                          }
                                                        };
                                                        await widget.socketApi
                                                            .sendRequest(
                                                                jsonRequest)
                                                            .then(
                                                                (re1sponse) => {
                                                                      Navigator.pop(
                                                                          context),
                                                                      requestPatientInfo(),
                                                                      _providerNameController
                                                                          .clear(),
                                                                      _providerIPController
                                                                          .clear(),
                                                                    });
                                                      },
                                                      child: const Text(
                                                          'Add Record'),
                                                    )
                                                  ])));
                                        });
                                  })
                              // Add list of record entries here
                            ],
                          ),
                          DataTable(
                            border: TableBorder.all(),
                            columns: const [
                              DataColumn(label: Text('Date')),
                              DataColumn(label: Text('Subject')),
                              DataColumn(label: Text('Provider')),
                            ],
                            rows:
                                (info['records'] as List<dynamic>).map((data) {
                              return DataRow(cells: [
                                DataCell(Text(data[0] ?? '')),
                                DataCell(Text(data[1] ?? '')),
                                DataCell(Text(data[2] ?? '')),
                              ]);
                            }).toList(),
                          )
                        ])),
              ),
            ],
          ),
        ));
  }
}
