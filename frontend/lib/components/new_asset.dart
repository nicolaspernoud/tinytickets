import 'package:flutter/material.dart';
import 'package:tinytickets/models/crud.dart';
import 'package:tinytickets/models/asset.dart';

import '../i18n.dart';

class NewEditAsset extends StatefulWidget {
  final Crud crud;
  final Asset asset;
  const NewEditAsset({Key? key, required this.crud, required this.asset})
      : super(key: key);

  @override
  _NewEditAssetState createState() => _NewEditAssetState();
}

class _NewEditAssetState extends State<NewEditAsset> {
  final _formKey = GlobalKey<FormState>();

  @override
  Widget build(BuildContext context) {
    // Build a Form widget using the _formKey created above.
    return Scaffold(
        appBar: AppBar(
          title: widget.asset.id > 0
              ? Text(MyLocalizations.of(context)!.tr("edit_asset"))
              : Text(MyLocalizations.of(context)!.tr("new_asset")),
          actions: widget.asset.id > 0
              ? [
                  IconButton(
                      icon: const Icon(Icons.delete_forever),
                      onPressed: () async {
                        await widget.crud.Delete(widget.asset.id);
                        Navigator.pop(context);
                        ScaffoldMessenger.of(context).showSnackBar(SnackBar(
                            content: Text(MyLocalizations.of(context)!
                                .tr("asset_deleted"))));
                      })
                ]
              : null,
        ),
        body: Padding(
            padding: const EdgeInsets.all(8.0),
            child: Form(
              key: _formKey,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  TextFormField(
                    initialValue: widget.asset.title,
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
                    onChanged: (value) {
                      widget.asset.title = value;
                    },
                  ),
                  TextFormField(
                    initialValue: widget.asset.description,
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
                    onChanged: (value) {
                      widget.asset.description = value;
                    },
                  ),
                  Padding(
                    padding: const EdgeInsets.symmetric(vertical: 16.0),
                    child: ElevatedButton(
                      onPressed: () async {
                        // Validate returns true if the form is valid, or false otherwise.
                        if (_formKey.currentState!.validate()) {
                          var msg =
                              MyLocalizations.of(context)!.tr("asset_created");
                          try {
                            if (widget.asset.id > 0) {
                              await widget.crud.Update(widget.asset);
                            } else {
                              await widget.crud.Create(widget.asset);
                            }
                            // Do nothing on TypeError as Create respond with a null id
                          } on TypeError {} catch (e) {
                            msg = e.toString();
                          }
                          ScaffoldMessenger.of(context).showSnackBar(
                            SnackBar(content: Text(msg)),
                          );
                          Navigator.pop(context);
                        }
                      },
                      child: Text(MyLocalizations.of(context)!.tr("submit")),
                    ),
                  ),
                ],
              ),
            )));
  }
}
