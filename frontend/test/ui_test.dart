import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:tinytickets/globals.dart';
import 'package:tinytickets/i18n.dart';
import 'package:tinytickets/main.dart';

Future<void> main() async {
  testWidgets('Basic app opening tests', (WidgetTester tester) async {
    // Initialize configuration
    SharedPreferences.setMockInitialValues({"hostname": "", "token": ""});
    await App().init();
    // Build our app and trigger a frame
    await tester.pumpWidget(
      MaterialApp(
        home: MyHomePage(title: 'Tiny Tickets'),
        localizationsDelegates: [
          const MyLocalizationsDelegate(),
        ],
      ),
    );

    // Check that the app title is displayed
    expect(find.text('Tiny Tickets'), findsOneWidget);
    await tester.pump();
    // Check that we do not display the ticket list on startup if a user token is not set
    expect(find.text("2021-08-12 - MyTicket"), findsNothing);
    // Enter a user token
    await tester.enterText(find.byKey(Key("tokenField")), '\$USER\$user');
    await tester.tap(find.text("OK"));
    await tester.pumpAndSettle();
    // Check that we display the ticket list if a user token is set
    expect(find.text("2021-08-12 - MyTicket"), findsOneWidget);

    // To print the widget tree :
    //debugDumpApp();
  });
}
