import 'package:flutter/material.dart';
import 'package:tinytickets/models/asset.dart';
import 'package:tinytickets/models/comment.dart';
import 'package:tinytickets/models/crud.dart';
import 'package:tinytickets/models/ticket.dart';

import '../globals.dart';
import '../i18n.dart';
import 'new_ticket.dart';
import 'settings.dart';

class Tickets extends StatefulWidget {
  final Crud crud;

  final String title;

  const Tickets({Key? key, required this.crud, required this.title})
      : super(key: key);

  @override
  _TicketsState createState() => _TicketsState();
}

class _TicketsState extends State<Tickets> {
  late Future<List<Ticket>> tickets;
  bool _showClosed = false;

  @override
  void initState() {
    super.initState();
    if (App().role != Role.unknown) tickets = widget.crud.ReadAll();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
        appBar: AppBar(
          title: Text(widget.title),
          actions: [
            IconButton(
                icon: const Icon(Icons.settings),
                onPressed: () async {
                  await Navigator.push(context,
                      MaterialPageRoute<void>(builder: (BuildContext context) {
                    return Settings(crud: APICrud<Asset>());
                  }));
                  setState(() {
                    if (App().role != Role.unknown)
                      tickets = widget.crud.ReadAll();
                  });
                })
          ],
        ),
        body: (App().role != Role.unknown)
            ? Center(
                child: Padding(
                padding: const EdgeInsets.all(16.0),
                child: FutureBuilder<List<Ticket>>(
                  future: tickets,
                  builder: (context, snapshot) {
                    if (snapshot.hasData) {
                      return ListView(
                          children: snapshot.data!
                              .where((t) => !t.is_closed || _showClosed)
                              .map((t) => Card(
                                      child: InkWell(
                                    splashColor: Colors.blue.withAlpha(30),
                                    onTap: () {
                                      _edit(t);
                                    },
                                    child: Column(
                                      mainAxisSize: MainAxisSize.min,
                                      children: <Widget>[
                                        ListTile(
                                            leading: Icon(Icons.assignment),
                                            title: Text(formatTime(t.time) +
                                                " - " +
                                                t.title),
                                            subtitle: Text(t.description))
                                      ],
                                    ),
                                  )))
                              .toList());
                    } else if (snapshot.hasError) {
                      return Text('${snapshot.error}');
                    }
                    // By default, show a loading spinner.
                    return const CircularProgressIndicator();
                  },
                ),
              ))
            : null,
        bottomNavigationBar: BottomAppBar(
            child: Padding(
          padding: const EdgeInsets.all(8),
          child: Row(
            children: [
              IconButton(
                  icon: const Icon(Icons.add),
                  onPressed: () {
                    _edit(Ticket(
                      id: 0,
                      title: "",
                      creator: "",
                      description: "",
                      asset_id: 1,
                      is_closed: false,
                      time: DateTime.now(),
                    ));
                  }),
              Expanded(
                child: Container(
                  height: 50,
                ),
              ),
              Row(
                mainAxisSize: MainAxisSize.min,
                children: <Widget>[
                  Text(MyLocalizations.of(context)!.tr("show_closed")),
                  Switch(
                    onChanged: (bool val) {
                      setState(() => _showClosed = val);
                    },
                    value: _showClosed,
                  ),
                ],
              ),
            ],
          ),
        )));
  }

  Future<void> _edit(t) async {
    await Navigator.of(context)
        .push(MaterialPageRoute<void>(builder: (BuildContext context) {
      return NewEditTicket(
          crud: APICrud<Ticket>(),
          assetsCrud: APICrud<Asset>(),
          commentsCrud: APICrud<Comment>(),
          ticket: t);
    }));
    setState(() {
      tickets = widget.crud.ReadAll();
    });
  }
}

String formatTime(DateTime d) {
  return "${d.year.toString()}-${d.month.toString().padLeft(2, "0")}-${d.day.toString().padLeft(2, "0")}";
}
