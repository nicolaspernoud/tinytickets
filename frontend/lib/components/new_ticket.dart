import 'dart:async';
import 'dart:math';
import 'dart:typed_data';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:image_picker/image_picker.dart';
import 'package:tinytickets/components/tickets.dart';
import 'package:tinytickets/models/asset.dart';
import 'package:tinytickets/models/comment.dart';
import 'package:tinytickets/models/crud.dart';
import 'package:tinytickets/models/ticket.dart';
import 'package:http/http.dart' as http;
import 'package:image/image.dart' as image;

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
  static const JPG_IMAGE_QUALITY = 80;

  final _formKey = GlobalKey<FormState>();
  late Future<Ticket> ticketWithComments;
  late bool isExisting;
  bool submitting = false;

  Future<Uint8List?>? imageBytes;
  String hostname = (App().prefs.getString("hostname") ?? "") + "/api";
  String token = App().prefs.getString("token") ?? "";

  double Function(BuildContext) thirdWidth = (BuildContext ctx) {
    var availableWidth = (MediaQuery.of(ctx).size.width - 32);
    var baseWidth = availableWidth / 3;
    if (baseWidth >= 250) {
      return baseWidth;
    } else {
      return availableWidth;
    }
  };

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

  _imgFromCamera() async {
    final temp = await ImagePicker().pickImage(
        source: ImageSource.camera,
        imageQuality: JPG_IMAGE_QUALITY,
        maxWidth: 1280);
    if (temp != null) {
      imageBytes = temp.readAsBytes();
      setState(() {});
    }
  }

  static Future<Uint8List> bakeOrientation(Uint8List img) async {
    final capturedImage = image.decodeImage(img);
    final orientedImage = image.bakeOrientation(capturedImage!);
    final encodedImage =
        image.encodeJpg(orientedImage, quality: JPG_IMAGE_QUALITY);
    return encodedImage as Uint8List;
  }

  Future<void> _imgToServer(int id) async {
    Uint8List? img = await imageBytes;
    if (imageBytes != null && img != null) {
      // Bake orientation on devices only as it is very slow and web does not support compute !!!
      if (!kIsWeb) {
        img = await compute(bakeOrientation, img);
      }
      final response = await http.post(
          Uri.parse('$hostname/tickets/photos/${id.toString()}'),
          headers: <String, String>{'X-TOKEN': token},
          body: img);
      if (response.statusCode != 200) {
        throw Exception(response.body.toString());
      }
    } else {
      await http.delete(
        Uri.parse('$hostname/tickets/photos/${id.toString()}'),
        headers: <String, String>{'X-TOKEN': token},
      );
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
    if (date != null)
      setState(() {
        widget.ticket.time = date;
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
        body: SingleChildScrollView(
            child: Padding(
          padding: const EdgeInsets.all(16.0),
          child: Form(
            key: _formKey,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                OutlinedButton(
                    onPressed: () async {
                      if (App().role == Role.admin) await _selectTime();
                    },
                    child: Padding(
                      padding: const EdgeInsets.all(16.0),
                      child: Text(formatTime(widget.ticket.time)),
                    )),
                SizedBox(height: 10),
                TextFormField(
                  readOnly: App().role != Role.admin && isExisting,
                  maxLength: 75,
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
                SizedBox(height: 10),
                AssetsDropDown(
                  crud: widget.assetsCrud,
                  callback: (val) => widget.ticket.asset_id = val,
                  initialIndex: widget.ticket.asset_id,
                ),
                SizedBox(height: 10),
                TextFormField(
                  readOnly: App().role != Role.admin && isExisting,
                  maxLines: 3,
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
                Wrap(
                  children: [
                    Container(
                      width: thirdWidth(context),
                      child: TextFormField(
                        readOnly: App().role != Role.admin && isExisting,
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
                    ),
                    Container(
                      width: thirdWidth(context),
                      child: TextFormField(
                        readOnly: App().role != Role.admin && isExisting,
                        decoration: new InputDecoration(
                            labelText: MyLocalizations.of(context)!
                                .tr("creator_mail")),
                        // The validator receives the text that the user has entered.
                        validator: (value) {
                          if (value == null ||
                              value.isEmpty ||
                              !value.isValidEmail()) {
                            return MyLocalizations.of(context)!
                                .tr("please_enter_valid_email");
                          }
                          return null;
                        },
                        initialValue: widget.ticket.creator_mail,
                        onChanged: (value) {
                          widget.ticket.creator_mail = value;
                        },
                      ),
                    ),
                    Container(
                      width: thirdWidth(context),
                      child: TextFormField(
                        readOnly: App().role != Role.admin && isExisting,
                        keyboardType: TextInputType.number,
                        inputFormatters: <TextInputFormatter>[
                          FilteringTextInputFormatter.allow(intOnly)
                        ],
                        decoration: new InputDecoration(
                            labelText: MyLocalizations.of(context)!
                                .tr("creator_phone")),
                        // The validator receives the text that the user has entered.
                        validator: (value) {
                          if (value == null ||
                              value.isEmpty ||
                              !value.isValidPhoneNumber()) {
                            return MyLocalizations.of(context)!
                                .tr("please_enter_valid_phone_number");
                          }
                          return null;
                        },
                        initialValue: widget.ticket.creator_phone,
                        onChanged: (value) {
                          widget.ticket.creator_phone = value;
                        },
                      ),
                    ),
                  ],
                ),
                SizedBox(height: 20),
                Center(
                  child: FutureBuilder<Uint8List?>(
                    future: imageBytes,
                    builder: (context, snapshot) {
                      if (snapshot.hasData && snapshot.data != null) {
                        return Row(
                          mainAxisAlignment: MainAxisAlignment.center,
                          children: [
                            InkWell(
                              onTap: () {
                                if (App().role == Role.admin || !isExisting)
                                  _imgFromCamera();
                              },
                              child: ClipRRect(
                                  borderRadius: BorderRadius.circular(20.0),
                                  child: Image.memory(
                                    snapshot.data!,
                                    fit: BoxFit.fitWidth,
                                    width: MediaQuery.of(context).size.width *
                                        0.75,
                                  )),
                            ),
                            if (App().role == Role.admin || !isExisting)
                              IconButton(
                                  onPressed: () {
                                    imageBytes = Future.value(null);
                                    setState(() {});
                                  },
                                  icon: Icon(Icons.clear))
                          ],
                        );
                      } else if (snapshot.hasError) {
                        return Text('${snapshot.error}');
                      }
                      return IconButton(
                          onPressed: () {
                            _imgFromCamera();
                          },
                          icon: Icon(Icons.camera_alt));
                    },
                  ),
                ),
                if (isExisting && App().role == Role.admin) ...[
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
                  )
                ],
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
                                              mainAxisSize: MainAxisSize.min,
                                              children: <Widget>[
                                                ListTile(
                                                  leading: Icon(Icons.comment),
                                                  title: Text(
                                                      formatTime(c.time) +
                                                          " - " +
                                                          c.creator),
                                                  subtitle: Text(c.content),
                                                  trailing:
                                                      App().role == Role.admin
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
                  SizedBox(
                    width: 140,
                    height: 50,
                    child: Center(
                      child: AnimatedSwitcher(
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
                        duration: Duration(milliseconds: 500),
                        child: !submitting
                            ? ElevatedButton(
                                onPressed: () async {
                                  // Validate returns true if the form is valid, or false otherwise.
                                  if (_formKey.currentState!.validate()) {
                                    submitting = true;
                                    setState(() {});
                                    var msg = MyLocalizations.of(context)!
                                        .tr("ticket_created");
                                    try {
                                      if (isExisting) {
                                        await widget.crud.Update(widget.ticket);
                                        await _imgToServer(widget.ticket.id);
                                      } else {
                                        var t = await widget.crud
                                            .Create(widget.ticket);
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
                                  child: Text(MyLocalizations.of(context)!
                                      .tr("submit")),
                                ),
                              )
                            : Center(child: CircularProgressIndicator()),
                      ),
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
            // Check that index exists
            var minID = snapshot.data!.first.id;
            var indexExists = false;
            for (final e in snapshot.data!) {
              if (e.id < minID) minID = e.id;
              if (_index == e.id) {
                indexExists = true;
                break;
              }
              ;
            }
            if (!indexExists) _index = minID;
            widget.callback(_index);
            return Row(
              children: [
                Text(MyLocalizations.of(context)!.tr("asset")),
                const SizedBox(width: 8),
                DropdownButton<int>(
                  value: _index,
                  items: snapshot.data!.map((a) {
                    return new DropdownMenuItem<int>(
                      value: a.id,
                      child: SizedBox(
                        width: max(MediaQuery.of(context).size.width / 2, 150),
                        child: new Text(
                          a.title,
                          overflow: TextOverflow.ellipsis,
                        ),
                      ),
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
          return Padding(
            padding: const EdgeInsets.all(16.0),
            child: Text(MyLocalizations.of(context)!.tr("no_assets")),
          );
        });
  }
}

extension EmailValidator on String {
  bool isValidEmail() {
    return RegExp(
            r'^(([^<>()[\]\\.,;:\s@\"]+(\.[^<>()[\]\\.,;:\s@\"]+)*)|(\".+\"))@((\[[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\])|(([a-zA-Z\-0-9]+\.)+[a-zA-Z]{2,}))$')
        .hasMatch(this);
  }
}

final intOnly = RegExp(r'^[0-9]*$');

extension PhoneValidator on String {
  bool isValidPhoneNumber() {
    return RegExp(r'^[0-9]{6,12}$').hasMatch(this);
  }
}
