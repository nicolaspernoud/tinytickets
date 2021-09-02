import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:tinytickets/globals.dart';
import 'package:tinytickets/i18n.dart';
import 'package:tinytickets/main.dart';

Future<void> main() async {
  testWidgets('Without token tests', (WidgetTester tester) async {
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
    // Check that we do not display the ticket list on startup if a desk token is not set
    expect(find.text("2021-08-12 - MyTicket"), findsNothing);
    await tester.tap(find.byIcon(Icons.settings));
    await tester.pumpAndSettle();
    // Enter a desk token
    await tester.enterText(find.byKey(Key("tokenField")), '\$DESK\$desk');
    await tester.pageBack();
    await tester.pumpAndSettle();
    // Check that we display the ticket list on startup if a desk token is set
    expect(find.text("2021-08-12 - MyTicket"), findsOneWidget);

    // To print the widget tree :
    //debugDumpApp();
  });
}
