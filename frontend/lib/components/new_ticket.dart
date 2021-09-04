import 'dart:typed_data';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:image_picker/image_picker.dart';
import 'package:tinytickets/components/tickets.dart';
import 'package:tinytickets/models/asset.dart';
import 'package:tinytickets/models/comment.dart';
import 'package:tinytickets/models/crud.dart';
import 'package:tinytickets/models/ticket.dart';
import 'package:http/http.dart' as http;

import '../globals.dart';
import '../i18n.dart';
import 'new_comment.dart';

class NewEditTicket extends StatefulWidget {
  final Crud crud;
  final Crud assetsCrud;
  final Crud commentsCrud;
  final Ticket ticket;
  const NewEditTicket(
      {Key? key,
      required this.crud,
      required this.assetsCrud,
      required this.commentsCrud,
      required this.ticket})
      : super(key: key);

  @override
  _NewEditTicketState createState() => _NewEditTicketState();
}

class _NewEditTicketState extends State<NewEditTicket> {
  final _formKey = GlobalKey<FormState>();
  late Future<Ticket> ticketWithComments;
  late bool isExisting;

  Future<Uint8List>? imageBytes;
  String hostname = (App().prefs.getString("hostname") ?? "") + "/api";
  String token = App().prefs.getString("token") ?? "";

  @override
  void initState() {
    super.initState();
    isExisting = widget.ticket.id > 0;
    if (isExisting) {
      ticketWithComments = widget.crud.Read(widget.ticket.id);
      _imgFromServer(widget.ticket.id);
    } else {
      ticketWithComments = Future<Ticket>.value(widget.ticket);
    }
  }

  _imgFromGallery() async {
    final temp = await ImagePicker().pickImage(
        source: ImageSource.camera, imageQuality: 80, maxWidth: 1280);
    imageBytes = temp!.readAsBytes();
    setState(() {});
  }

  _imgToServer(int id) async {
    if (imageBytes != null) {
      final response = await http.post(
        Uri.parse('$hostname/tickets/photos/${id.toString()}'),
        headers: <String, String>{'X-TOKEN': token},
        body: await imageBytes,
      );
      if (response.statusCode != 200) {
        throw Exception(response.body.toString());
      }
    }
  }

  _imgFromServer(int id) async {
    final response = await http.get(
      Uri.parse('$hostname/tickets/photos/${id.toString()}'),
      headers: <String, String>{'X-TOKEN': token},
    );
    if (response.statusCode == 200) {
      imageBytes = Future.value(response.bodyBytes);
      setState(() {});
    }
  }

  // Time selector
  Future<void> _selectTime() async {
    final date = await showDatePicker(
        context: context,
        initialDate: widget.ticket.time,
        firstDate: DateTime(2014),
        lastDate: DateTime.now().add(Duration(days: 30)));
    setState(() {
      widget.ticket.time = date!;
    });
  }

  @override
  Widget build(BuildContext context) {
    // Build a Form widget using the _formKey created above.
    return Scaffold(
        appBar: AppBar(
          title: isExisting
              ? Text(MyLocalizations.of(context)!.tr("edit_ticket"))
              : Text(MyLocalizations.of(context)!.tr("new_ticket")),
          actions: (isExisting && App().role == Role.admin)
              ? [
                  IconButton(
                      icon: const Icon(Icons.delete_forever),
                      onPressed: () async {
                        await widget.crud.Delete(widget.ticket.id);
                        Navigator.pop(context);
                        ScaffoldMessenger.of(context).showSnackBar(SnackBar(
                            content: Text(MyLocalizations.of(context)!
                                .tr("ticket_deleted"))));
                      })
                ]
              : null,
        ),
        body: Padding(
            padding: const EdgeInsets.all(16.0),
            child: SingleChildScrollView(
              child: Form(
                key: _formKey,
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    OutlinedButton(
                        onPressed: () async {
                          await _selectTime();
                        },
                        child: Padding(
                          padding: const EdgeInsets.all(16.0),
                          child: Text(formatTime(widget.ticket.time)),
                        )),
                    TextFormField(
                      decoration: new InputDecoration(
                          labelText: MyLocalizations.of(context)!.tr("title")),
                      // The validator receives the text that the user has entered.
                      validator: (value) {
                        if (value == null || value.isEmpty) {
                          return MyLocalizations.of(context)!
                              .tr("please_enter_some_text");
                        }
                        return null;
                      },
                      initialValue: widget.ticket.title,
                      onChanged: (value) {
                        widget.ticket.title = value;
                      },
                    ),
                    TextFormField(
                      decoration: new InputDecoration(
                          labelText:
                              MyLocalizations.of(context)!.tr("creator")),
                      // The validator receives the text that the user has entered.
                      validator: (value) {
                        if (value == null || value.isEmpty) {
                          return MyLocalizations.of(context)!
                              .tr("please_enter_some_text");
                        }
                        return null;
                      },
                      initialValue: widget.ticket.creator,
                      onChanged: (value) {
                        widget.ticket.creator = value;
                      },
                    ),
                    TextFormField(
                      decoration: new InputDecoration(
                          labelText:
                              MyLocalizations.of(context)!.tr("description")),
                      // The validator receives the text that the user has entered.
                      validator: (value) {
                        if (value == null || value.isEmpty) {
                          return MyLocalizations.of(context)!
                              .tr("please_enter_some_text");
                        }
                        return null;
                      },
                      initialValue: widget.ticket.description,
                      onChanged: (value) {
                        widget.ticket.description = value;
                      },
                    ),
                    SizedBox(height: 20),
                    Center(
                      child: FutureBuilder<Uint8List>(
                        future: imageBytes,
                        builder: (context, snapshot) {
                          if (snapshot.hasData && snapshot.data != null) {
                            return InkWell(
                              onTap: () {
                                _imgFromGallery();
                              },
                              child: ClipRRect(
                                  borderRadius: BorderRadius.circular(20.0),
                                  child: Image.memory(
                                    snapshot.data!,
                                    fit: BoxFit.fill,
                                    height: 300,
                                  )),
                            );
                          } else if (snapshot.hasError) {
                            return Text('${snapshot.error}');
                          }
                          return IconButton(
                              onPressed: () {
                                _imgFromGallery();
                              },
                              icon: Icon(Icons.camera_alt));
                        },
                      ),
                    ),
                    AssetsDropDown(
                      crud: widget.assetsCrud,
                      callback: (val) => widget.ticket.asset_id = val,
                      initialIndex: widget.ticket.asset_id,
                    ),
                    SizedBox(height: 20),
                    Row(
                      children: [
                        Text(MyLocalizations.of(context)!.tr("closed")),
                        Switch(
                            value: widget.ticket.is_closed,
                            onChanged: (v) => setState(() {
                                  widget.ticket.is_closed = v;
                                })),
                      ],
                    ),
                    SizedBox(height: 20),
                    if (isExisting)
                      Center(
                        child: Column(
                          children: [
                            Text(
                              MyLocalizations.of(context)!.tr("comments"),
                              style: TextStyle(fontWeight: FontWeight.bold),
                            ),
                            SizedBox(height: 20),
                            FutureBuilder<Ticket>(
                              future: ticketWithComments,
                              builder: (context, snapshot) {
                                if (snapshot.hasData) {
                                  return Column(
                                    children: [
                                      ...snapshot.data!.comments
                                          .map((c) => Card(
                                                  child: InkWell(
                                                splashColor:
                                                    Colors.blue.withAlpha(30),
                                                onTap: () {
                                                  _edit(c);
                                                },
                                                child: Column(
                                                  mainAxisSize:
                                                      MainAxisSize.min,
                                                  children: <Widget>[
                                                    ListTile(
                                                      leading:
                                                          Icon(Icons.comment),
                                                      title: Text(
                                                          formatTime(c.time) +
                                                              " - " +
                                                              c.creator),
                                                      subtitle: Text(c.content),
                                                      trailing: App().role ==
                                                              Role.admin
                                                          ? IconButton(
                                                              icon: Icon(Icons
                                                                  .delete_forever),
                                                              onPressed: () {
                                                                _delete(c);
                                                              },
                                                            )
                                                          : null,
                                                    ),
                                                  ],
                                                ),
                                              )))
                                          .toList(),
                                      Padding(
                                        padding: const EdgeInsets.all(16.0),
                                        child: IconButton(
                                          icon: const Icon(Icons.add_comment),
                                          color: Colors.blue,
                                          onPressed: () {
                                            _edit(Comment(
                                                id: 0,
                                                ticket_id: widget.ticket.id,
                                                creator: "",
                                                content: "",
                                                time: DateTime.now()));
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
                            ),
                          ],
                        ),
                      ),
                    SizedBox(height: 20),
                    if (App().role == Role.admin || !isExisting)
                      ElevatedButton(
                        onPressed: () async {
                          // Validate returns true if the form is valid, or false otherwise.
                          if (_formKey.currentState!.validate()) {
                            var msg = MyLocalizations.of(context)!
                                .tr("ticket_created");
                            try {
                              if (isExisting) {
                                await widget.crud.Update(widget.ticket);
                                await _imgToServer(widget.ticket.id);
                              } else {
                                var t = await widget.crud.Create(widget.ticket);
                                await _imgToServer(t.id);
                              }
                            } catch (e) {
                              msg = e.toString();
                            }
                            ScaffoldMessenger.of(context).showSnackBar(
                              SnackBar(content: Text(msg)),
                            );
                            Navigator.pop(context);
                          }
                        },
                        child: Padding(
                          padding: const EdgeInsets.all(16.0),
                          child:
                              Text(MyLocalizations.of(context)!.tr("submit")),
                        ),
                      ),
                  ],
                ),
              ),
            )));
  }

  Future<void> _edit(Comment c) async {
    await Navigator.of(context)
        .push(MaterialPageRoute<void>(builder: (BuildContext context) {
      return NewEditComment(crud: widget.commentsCrud, comment: c);
    }));
    setState(() {
      ticketWithComments = widget.crud.Read(widget.ticket.id);
    });
  }

  Future<void> _delete(Comment c) async {
    await widget.commentsCrud.Delete(c.id);
    setState(() {
      ticketWithComments = widget.crud.Read(widget.ticket.id);
    });
  }
}

typedef void IntCallback(int val);

class AssetsDropDown extends StatefulWidget {
  final IntCallback callback;
  final Crud crud;
  final int initialIndex;
  const AssetsDropDown({
    Key? key,
    required this.crud,
    required this.callback,
    required this.initialIndex,
  }) : super(key: key);

  @override
  _AssetsDropDownState createState() => _AssetsDropDownState();
}

class _AssetsDropDownState extends State<AssetsDropDown> {
  late Future<List<Asset>> assets;
  late int _index;

  @override
  void initState() {
    super.initState();
    assets = widget.crud.ReadAll();
    _index = widget.initialIndex;
  }

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<List<Asset>>(
        future: assets,
        builder: (context, snapshot) {
          if (snapshot.hasData && snapshot.data!.length > 0) {
            return Row(
              children: [
                Text(MyLocalizations.of(context)!.tr("asset")),
                const SizedBox(width: 8),
                DropdownButton<int>(
                  value: _index,
                  items: snapshot.data!.map((a) {
                    return new DropdownMenuItem<int>(
                      value: a.id,
                      child: new Text(a.title),
                    );
                  }).toList(),
                  onChanged: (value) {
                    setState(() {
                      _index = value!;
                    });
                    widget.callback(value!);
                  },
                ),
              ],
            );
          } else if (snapshot.hasError) {
            return Text('${snapshot.error}');
          }
          // By default, show a loading spinner.
          return Padding(
            padding: const EdgeInsets.all(16.0),
            child: Text(MyLocalizations.of(context)!.tr("no_assets")),
          );
        });
  }
}
