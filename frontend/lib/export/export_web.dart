import 'package:tinytickets/globals.dart';
import 'package:http/http.dart' as http;
import 'package:web/web.dart' as web;
import 'dart:js_interop';

// JS interop for URL.createObjectURL and revokeObjectURL
@JS('URL.createObjectURL')
external String createObjectURL(web.Blob blob);

@JS('URL.revokeObjectURL')
external void revokeObjectURL(String url);

Future<void> export() async {
  String base = (App().prefs.getString("hostname") ?? "") + "/api";
  String token = App().prefs.getString("token") ?? "";

  final response = await http.get(
    Uri.parse('$base/tickets/export'),
    headers: <String, String>{
      'X-TOKEN': token,
    },
  );

  if (response.statusCode == 200) {
    final blob = web.Blob(<web.BlobPart>[response.bodyBytes.toJS].toJS);

    final url = createObjectURL(blob);

    final anchor = web.document.createElement('a') as web.HTMLAnchorElement;
    anchor
      ..href = url
      ..download = 'tiny_tickets_export.html'
      ..style.display = 'none';

    web.document.body!.appendChild(anchor);
    anchor.click();
    web.document.body!.removeChild(anchor);

    revokeObjectURL(url);
  } else {
    throw Exception('Failed to export tickets');
  }
}
