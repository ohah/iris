#import "IrisBenchTurboModule.h"

#import <React/RCTBridgeModule.h>

#import <math.h>

#import <memory>

using namespace facebook::react;

@implementation RCTIrisBenchTurboModule

RCT_EXPORT_MODULE(IrisBenchTurboModule)

+ (BOOL)requiresMainQueueSetup
{
  return NO;
}

- (std::shared_ptr<facebook::react::TurboModule>)getTurboModule:
    (const facebook::react::ObjCTurboModule::InitParams &)params
{
  return std::make_shared<NativeIrisBenchTurboModuleSpecJSI>(params);
}

RCT_EXPORT_SYNCHRONOUS_TYPED_METHOD(NSNumber *, echoNumber : (double)value)
{
  return @(value);
}

RCT_EXPORT_SYNCHRONOUS_TYPED_METHOD(NSString *, getIrisRuntimeLabel)
{
  return @"iris-ios-module";
}

RCT_EXPORT_SYNCHRONOUS_TYPED_METHOD(NSString *, getRuntimeLabel)
{
  return @"ios-turbomodule";
}

RCT_EXPORT_METHOD(noop)
{
}

RCT_EXPORT_SYNCHRONOUS_TYPED_METHOD(NSString *, roundTripString : (NSString *)value)
{
  return value;
}

RCT_EXPORT_SYNCHRONOUS_TYPED_METHOD(NSNumber *, runIrisNumericWorkload : (double)iterations)
{
  NSInteger boundedIterations = (NSInteger)iterations;
  if (boundedIterations < 0) {
    boundedIterations = 0;
  }

  double checksum = 0;
  for (NSInteger index = 0; index < boundedIterations; index += 1) {
    checksum += sqrt((double)((index % 1000) + 1)) * sin((double)index);
  }

  return @(round(checksum * 1000.0) / 1000.0);
}

@end
