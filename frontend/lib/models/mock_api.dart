import 'package:http/http.dart';
import 'package:http/testing.dart';

class MockAPI {
  late final Client client;
  MockAPI() {
    client = MockClient((request) async {
      switch (request.url.toString()) {
        case '/api/tickets/all':
          return Response(
              '[{"id":1,"asset_id":1,"title":"MyTicket","creator":"A Creator","description":"MyDescription","time":"2021-08-12T20:00:00","is_closed":false}]',
              200);
        default:
          return Response('Not Found', 404);
      }
    });
  }
}
