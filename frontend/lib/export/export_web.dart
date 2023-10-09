import 'package:tinytickets/globals.dart';
import 'package:http/http.dart' as http;
import 'dart:html' as html;

export() async {
  String base = (App().prefs.getString("hostname") ?? "") + "/api";
  String token = App().prefs.getString("token") ?? "";

  final response = await http.get(
    Uri.parse('$base/tickets/export'),
    headers: <String, String>{
      'X-TOKEN': token,
    },
  );
  if (response.statusCode == 200) {
    final blob = html.Blob([response.bodyBytes]);
    final url = html.Url.createObjectUrlFromBlob(blob);
    final anchor = html.document.createElement('a') as html.AnchorElement
      ..href = url
      ..style.display = 'none'
      ..download = "tiny_tickets_export.html";
    html.document.body!.children.add(anchor);

    anchor.click();

    html.document.body!.children.remove(anchor);
    html.Url.revokeObjectUrl(url);
  } else {
    throw Exception('Failed to export tickets');
  }
}
