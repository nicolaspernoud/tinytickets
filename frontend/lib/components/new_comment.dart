import 'package:flutter/material.dart';
import 'package:tinytickets/models/crud.dart';
import 'package:tinytickets/models/comment.dart';

import '../globals.dart';
import '../i18n.dart';

class NewEditComment extends StatefulWidget {
  final Crud crud;
  final Comment comment;
  const NewEditComment({Key? key, required this.crud, required this.comment})
      : super(key: key);

  @override
  _NewEditCommentState createState() => _NewEditCommentState();
}

class _NewEditCommentState extends State<NewEditComment> {
  final _formKey = GlobalKey<FormState>();

  @override
  Widget build(BuildContext context) {
    // Build a Form widget using the _formKey created above.
    return Scaffold(
        appBar: AppBar(
          title: widget.comment.id > 0
              ? Text(MyLocalizations.of(context)!.tr("edit_comment"))
              : Text(MyLocalizations.of(context)!.tr("new_comment")),
          actions: (widget.comment.id > 0 && App().role == Role.admin)
              ? [
                  IconButton(
                      icon: const Icon(Icons.delete_forever),
                      onPressed: () async {
                        await widget.crud.Delete(widget.comment.id);
                        Navigator.pop(context);
                        ScaffoldMessenger.of(context).showSnackBar(SnackBar(
                            content: Text(MyLocalizations.of(context)!
                                .tr("comment_deleted"))));
                      })
                ]
              : null,
        ),
        body: Padding(
            padding: const EdgeInsets.all(16.0),
            child: Form(
              key: _formKey,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  TextFormField(
                    initialValue: widget.comment.creator,
                    decoration: new InputDecoration(
                        labelText: MyLocalizations.of(context)!.tr("creator")),
                    // The validator receives the text that the user has entered.
                    validator: (value) {
                      if (value == null || value.isEmpty) {
                        return MyLocalizations.of(context)!
                            .tr("please_enter_some_text");
                      }
                      return null;
                    },
                    onChanged: (value) {
                      widget.comment.creator = value;
                    },
                  ),
                  TextFormField(
                    initialValue: widget.comment.content,
                    decoration: new InputDecoration(
                        labelText: MyLocalizations.of(context)!.tr("content")),
                    // The validator receives the text that the user has entered.
                    validator: (value) {
                      if (value == null || value.isEmpty) {
                        return MyLocalizations.of(context)!
                            .tr("please_enter_some_text");
                      }
                      return null;
                    },
                    onChanged: (value) {
                      widget.comment.content = value;
                    },
                  ),
                  if (App().role == Role.admin || widget.comment.id == 0)
                    Padding(
                      padding: const EdgeInsets.symmetric(vertical: 16.0),
                      child: ElevatedButton(
                        onPressed: () async {
                          // Validate returns true if the form is valid, or false otherwise.
                          if (_formKey.currentState!.validate()) {
                            var msg = MyLocalizations.of(context)!
                                .tr("comment_created");
                            try {
                              if (widget.comment.id > 0) {
                                await widget.crud.Update(widget.comment);
                              } else {
                                await widget.crud.Create(widget.comment);
                              }
                            } on TypeError {} catch (e) {
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
                    ),
                ],
              ),
            )));
  }
}
