import 'package:flutter/material.dart';
import '../i18n.dart';

class DeleteDialog extends StatefulWidget {
  const DeleteDialog({super.key});

  @override
  State<DeleteDialog> createState() => _DeleteDialogState();
}

class _DeleteDialogState extends State<DeleteDialog> {
  late bool confirmed;

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: Column(
        children: [
          Text(
              "${MyLocalizations.of(context)!.tr("confirm_deletion")} ?"),
          const SizedBox(height: 15),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceEvenly,
            children: [
              ElevatedButton(
                style: ElevatedButton.styleFrom(
                    backgroundColor: Colors.red, foregroundColor: Colors.black),
                onPressed: () => Navigator.pop(context, true),
                child: Padding(
                  padding: const EdgeInsets.all(8.0),
                  child: Text(MyLocalizations.of(context)!.tr("delete")),
                ),
              ),
              Spacer(flex: 2),
              ElevatedButton(
                style: ElevatedButton.styleFrom(
                  backgroundColor: Colors.green,
                  foregroundColor: Colors.black,
                ),
                onPressed: () => Navigator.pop(context, false),
                child: Padding(
                  padding: const EdgeInsets.all(8.0),
                  child: Text(MyLocalizations.of(context)!.tr("cancel")),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class DeletingSpinner extends StatelessWidget {
  const DeletingSpinner({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return const SizedBox(
      width: 30,
      height: 30,
      child: CircularProgressIndicator(
          valueColor: AlwaysStoppedAnimation<Color>(Colors.grey)),
    );
  }
}
