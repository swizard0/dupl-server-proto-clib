package AIP::Kribrum::HashDuplClient;

use strict;
use vars qw($AUTOLOAD $VERSION $ABSTRACT @ISA @EXPORT);

BEGIN {
	$VERSION = 0.1;
	$ABSTRACT = "HashDuplClient. Ashmanov & Partners, Kribrum";
	
	@ISA = qw(Exporter DynaLoader);
	@EXPORT = qw(
	);
};

bootstrap AIP::Kribrum::HashDuplClient $VERSION;

use DynaLoader ();
use Exporter ();

1;


__END__

=head1 NAME

AIP::Kribrum::HashDuplClient - HashDupl. Ashmanov & Partners, Kribrum

=head1 SYNOPSIS

=head1 METHODS

=head2 new

=head1 DESTROY

 undef $obj;

Освобождает все занятые ресурсы, уничтожает объект.

=head1 AUTHOR

Alexander Borisov <lex.borisov@gmail.com>

=head1 COPYRIGHT AND LICENSE

This software is copyright (c) 2014 by Ashmanov & Partners

This is proprietary software; internal usage only.

=cut

#<style>pre {border: 1px solid #aaa; background-color: #ddd; padding: 10px 10px 10px 10px;}</style>