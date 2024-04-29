import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import 'package:client/socket_api.dart';

class RecordPage extends StatefulWidget {
  const RecordPage(
      {super.key,
      required this.socketApi,
      required this.id,
      required this.blockId});

  final SocketApi socketApi;
  final String id;
  final int blockId;

  @override
  State<RecordPage> createState() => _RecordPage();
}

class _RecordPage extends State<RecordPage> {
  Map record = {"timestamp": 0, "subject": "", "text": ""};

  void requestRecord() async {
    Map<String, dynamic> jsonRequest = {
      'action': 'get_record',
      'parameters': {'id': widget.id, 'block_id': widget.blockId}
    };
    dynamic fetchedRecord = await widget.socketApi.sendRequest(jsonRequest);
    print(fetchedRecord);
    DateTime dateTime =
        DateTime.fromMillisecondsSinceEpoch(fetchedRecord["timestamp"] * 1000);
    String dateDisplay = DateFormat.yMMMMd().format(dateTime);

    setState(() {
      record = {
        "dateDisplay": dateDisplay,
        "subject": fetchedRecord["subject"],
        "text": fetchedRecord["text"],
      };
    });
  }

  @override
  void initState() {
    super.initState();
    requestRecord();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
        appBar: AppBar(),
        body: Container(
            padding: const EdgeInsets.all(20),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text('Date:     ${record["dateDisplay"]}'),
                Text('Subject:  ${record["subject"]}'),
                Text('Notes:    ${record["text"]}')
              ],
            )));
  }
}
