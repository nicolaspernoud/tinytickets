import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:tinytickets/models/asset.dart';
import 'package:tinytickets/models/comment.dart';
import 'package:tinytickets/models/crud.dart';
import 'package:tinytickets/models/ticket.dart';
import 'package:http/http.dart' as http;

import '../globals.dart';
import '../i18n.dart';
import 'new_ticket.dart';
import 'settings.dart';

import 'package:tinytickets/export/export_android.dart'
    if (dart.library.js) 'package:tinytickets/export/export_web.dart';

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
  late Future<String> app_title;
  bool _showClosed = false;
  String _titleFilter = "";

  @override
  void initState() {
    super.initState();
    app_title = http
        .get(
          Uri.parse(
              (App().prefs.getString("hostname") ?? "") + "/api/app-title"),
        )
        .then((value) => value.statusCode == 200 ? value.body : "Tiny Tickets");
    if (App().role != Role.unknown) {
      tickets = widget.crud.ReadAll();
    } else {
      WidgetsBinding.instance.addPostFrameCallback(openSettings);
    }
    ;
  }

  void openSettings(_) async {
    await showDialog<String>(
      context: context,
      builder: (BuildContext context) => AlertDialog(
        title: Text(MyLocalizations.of(context)!.tr("settings")),
        content: Container(
          child: const settingsField(),
          height: 150,
        ),
        actions: <Widget>[
          TextButton(
            onPressed: () => Navigator.pop(context, 'OK'),
            child: const Text('OK'),
          ),
        ],
      ),
    );
    setState(() {
      hasRoleOrOpenSettings(_);
    });
  }

  void hasRoleOrOpenSettings(_) {
    if (App().role != Role.unknown) {
      tickets = widget.crud.ReadAll();
    } else {
      openSettings(_);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
        appBar: AppBar(
          title: Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Image.asset(
                'assets/icon/icon.png',
                fit: BoxFit.contain,
                height: 40,
              ),
              const SizedBox(width: 8),
              Container(
                  child: FutureBuilder<String>(
                future: app_title,
                builder: (context, snapshot) {
                  Widget child;
                  if (snapshot.hasData) {
                    child = Text(snapshot.data!,
                        style: TextStyle(fontWeight: FontWeight.bold),
                        key: ValueKey<int>(1));
                  } else {
                    child = Text("Tiny Tickets",
                        style: TextStyle(fontWeight: FontWeight.bold),
                        key: ValueKey<int>(0));
                  }
                  return AnimatedSwitcher(
                    duration: Duration(milliseconds: 1500),
                    child: child,
                    switchInCurve: Interval(
                      0.5,
                      1,
                      curve: Curves.linear,
                    ),
                    switchOutCurve: Interval(
                      0,
                      0.5,
                      curve: Curves.linear,
                    ).flipped,
                  );
                },
              )),
            ],
          ),
          actions: [
            IconButton(
                icon: const Icon(Icons.settings),
                onPressed: () async {
                  await Navigator.push(context,
                      MaterialPageRoute<void>(builder: (BuildContext context) {
                    return Settings(crud: APICrud<Asset>());
                  }));
                  setState(() {
                    hasRoleOrOpenSettings(null);
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
                    Widget child;
                    if (snapshot.hasData) {
                      var ts = snapshot.data!.where((t) =>
                          (!t.is_closed || _showClosed) &&
                          t.title
                              .toLowerCase()
                              .contains(_titleFilter.toLowerCase()));
                      child = RefreshIndicator(
                        color: Colors.amberAccent,
                        onRefresh: () {
                          tickets = widget.crud.ReadAll();
                          setState(() {});
                          return tickets;
                        },
                        child: ListView.builder(
                          itemBuilder: (ctx, i) {
                            return Card(
                                child: InkWell(
                              splashColor: Colors.amber.withAlpha(30),
                              onTap: () {
                                _edit(ts.elementAt(i));
                              },
                              child: Column(
                                mainAxisSize: MainAxisSize.min,
                                children: <Widget>[
                                  ListTile(
                                      leading: Icon(ts.elementAt(i).is_closed
                                          ? Icons.assignment_turned_in
                                          : Icons.assignment),
                                      title: Text(
                                          formatTime(ts.elementAt(i).time) +
                                              " - " +
                                              ts.elementAt(i).title),
                                      subtitle: Text(
                                        ts.elementAt(i).description,
                                        maxLines: 2,
                                      ))
                                ],
                              ),
                            ));
                          },
                          itemCount: ts.length,
                          physics: const AlwaysScrollableScrollPhysics(),
                        ),
                      );
                    } else if (snapshot.hasError) {
                      child = Text(
                          MyLocalizations.of(context)!.tr("try_new_token"));
                    } else {
                      child = const CircularProgressIndicator();
                    }
                    return AnimatedSwitcher(
                      duration: Duration(milliseconds: 300),
                      child: child,
                    );
                  },
                ),
              ))
            : null,
        bottomNavigationBar: BottomAppBar(
            child: Wrap(
          direction: Axis.horizontal,
          alignment: WrapAlignment.spaceBetween,
          crossAxisAlignment: WrapCrossAlignment.center,
          children: [
            Wrap(
              crossAxisAlignment: WrapCrossAlignment.center,
              children: [
                IconButton(
                    icon: const Icon(Icons.add),
                    onPressed: () {
                      _edit(Ticket(
                        id: 0,
                        title: "",
                        creator: "",
                        creator_mail: "",
                        creator_phone: "",
                        description: "",
                        asset_id: 1,
                        is_closed: false,
                        time: DateTime.now(),
                      ));
                    }),
                Text(MyLocalizations.of(context)!.tr("create_ticket")),
              ],
            ),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 8.0),
              child: Wrap(
                crossAxisAlignment: WrapCrossAlignment.center,
                children: [
                  const Padding(
                      padding: const EdgeInsets.only(right: 8.0),
                      child: Icon(Icons.search)),
                  SizedBox(
                    width: 150,
                    child: TextFormField(
                        initialValue: _titleFilter,
                        decoration: InputDecoration(
                            labelText:
                                MyLocalizations.of(context)!.tr("search")),
                        onChanged: (value) {
                          setState(() {
                            _titleFilter = value;
                          });
                        }),
                  ),
                ],
              ),
            ),
            Padding(
              padding: const EdgeInsets.only(left: 12.0),
              child: Wrap(
                crossAxisAlignment: WrapCrossAlignment.center,
                children: [
                  Text(MyLocalizations.of(context)!.tr("show_closed")),
                  Switch(
                    onChanged: (bool val) {
                      setState(() => _showClosed = val);
                    },
                    value: _showClosed,
                  ),
                ],
              ),
            ),
            if (kIsWeb)
              IconButton(
                  icon: const Icon(Icons.download), onPressed: () => export()),
          ],
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
