import 'package:flutter/material.dart';
import 'package:tinytickets/components/new_asset.dart';
import 'package:tinytickets/models/asset.dart';
import 'package:tinytickets/models/crud.dart';
import 'package:flutter/foundation.dart' show kDebugMode, kIsWeb;

import '../globals.dart';
import '../i18n.dart';

class Settings extends StatefulWidget {
  final Crud crud;
  const Settings({Key? key, required this.crud}) : super(key: key);

  @override
  _SettingsState createState() => _SettingsState();
}

class _SettingsState extends State<Settings> {
  late Future<List<Asset>> assets;

  @override
  void initState() {
    super.initState();
    if (App().role == Role.admin) assets = widget.crud.ReadAll();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
        appBar: AppBar(
          title: Text(MyLocalizations.of(context)!.tr("settings")),
        ),
        body: Center(
            child: Padding(
          padding: const EdgeInsets.all(16.0),
          child: Column(
            children: [
              settingsField(),
              if (App().role == Role.admin) ...[
                Padding(
                  padding: const EdgeInsets.all(20),
                  child: Text(
                    MyLocalizations.of(context)!.tr("assets"),
                    style: TextStyle(fontWeight: FontWeight.bold),
                  ),
                ),
                FutureBuilder<List<Asset>>(
                  future: assets,
                  builder: (context, snapshot) {
                    if (snapshot.hasData) {
                      return Column(
                        children: [
                          ...snapshot.data!
                              .map((a) => Card(
                                      child: InkWell(
                                    splashColor: Colors.blue.withAlpha(30),
                                    onTap: () {
                                      _edit(a);
                                    },
                                    child: Column(
                                      mainAxisSize: MainAxisSize.min,
                                      children: <Widget>[
                                        ListTile(
                                          leading: Icon(Icons.apartment),
                                          title: Text(a.title),
                                          subtitle: Text(a.description),
                                        ),
                                      ],
                                    ),
                                  )))
                              .toList(),
                          Padding(
                            padding: const EdgeInsets.all(16.0),
                            child: IconButton(
                              icon: const Icon(Icons.add),
                              color: Colors.blue,
                              onPressed: () {
                                _edit(Asset(id: 0, title: "", description: ""));
                              },
                            ),
                          ),
                        ],
                      );
                    } else if (snapshot.hasError) {
                      return Text('${snapshot.error}');
                    }
                    // By default, show a loading spinner.
                    return const CircularProgressIndicator();
                  },
                )
              ],
            ],
          ),
        )));
  }

  Future<void> _edit(Asset a) async {
    await Navigator.of(context)
        .push(MaterialPageRoute<void>(builder: (BuildContext context) {
      return NewEditAsset(crud: APICrud<Asset>(), asset: a);
    }));
    setState(() {
      assets = widget.crud.ReadAll();
    });
  }
}

class settingsField extends StatelessWidget {
  const settingsField({
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        if (!kIsWeb || kDebugMode)
          TextFormField(
            initialValue: App().prefs.getString("hostname"),
            decoration: new InputDecoration(
                labelText: MyLocalizations.of(context)!.tr("hostname")),
            onChanged: (text) {
              App().prefs.setString("hostname", text);
            },
            key: Key("hostnameField"),
          ),
        SizedBox(height: 20),
        TextFormField(
          initialValue: App().prefs.getString("token"),
          decoration: new InputDecoration(
              labelText: MyLocalizations.of(context)!.tr("token")),
          onChanged: (text) {
            App().prefs.setString("token", text);
          },
          key: Key("tokenField"),
        ),
      ],
    );
  }
}
